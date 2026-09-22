//! The set of tracked directories. The filesystem is the source of truth: a directory is
//! tracked if and only if it lives in the root and is named `<id>` or `<id>-<label>`.

use std::fs::{self, DirBuilder, File};
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};

use crate::error::{Error, IoContext, Result};
use crate::fsx;
use crate::name::{DirName, DirRef, Label};

/// A tracked temporary directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedDir {
    pub name: DirName,
    pub path: PathBuf,
}

impl TrackedDir {
    /// Deletes the directory and everything in it.
    pub fn remove(&self) -> Result<()> {
        fs::remove_dir_all(&self.path).context("remove", &self.path)
    }

    /// Moves the directory out of the store and returns its new path.
    ///
    /// Like `mv`, an existing directory `destination` receives the saved directory, named
    /// after its label (or `tempit-<id>`); any other path becomes the new path. Nothing is
    /// ever overwritten.
    pub fn save_to(&self, destination: &Path) -> Result<PathBuf> {
        let target = if destination.is_dir() {
            destination.join(self.saved_name())
        } else {
            destination.to_path_buf()
        };
        if let Some(parent) = target.parent() {
            let parent = fs::canonicalize(parent).context("resolve", parent)?;
            let source = fs::canonicalize(&self.path).context("resolve", &self.path)?;
            if parent.starts_with(&source) {
                return Err(Error::SaveIntoItself(self.path.clone()));
            }
        }
        fsx::move_dir(&self.path, &target)?;
        Ok(target)
    }

    fn saved_name(&self) -> String {
        match &self.name.label {
            Some(label) => label.to_string(),
            None => format!("tempit-{}", self.name.id),
        }
    }
}

/// The root directory holding the tracked directories.
#[derive(Debug)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Opens the store at `root`, creating it (private to the current user) if needed.
    ///
    /// An existing root must belong to the current user, so that another user of a shared
    /// `/tmp` cannot plant a directory or symlink there to capture our files.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&root)
            .context("create", &root)?;
        let owner = fs::symlink_metadata(&root).context("inspect", &root)?.uid();
        if owner != rustix::process::getuid().as_raw() {
            return Err(Error::ForeignRoot(root));
        }
        if !root.is_dir() {
            return Err(Error::NotADirectory(root));
        }
        Ok(Self { root })
    }

    /// All tracked directories, sorted by id.
    pub fn list(&self) -> Result<Vec<TrackedDir>> {
        let mut dirs = Vec::new();
        for entry in fs::read_dir(&self.root).context("read", &self.root)? {
            let entry = entry.context("read", &self.root)?;
            let Some(name) = entry.file_name().to_str().and_then(DirName::parse) else {
                continue;
            };
            // Symlinks are deliberately not followed: only real directories are tracked.
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                dirs.push(TrackedDir {
                    name,
                    path: entry.path(),
                });
            }
        }
        dirs.sort_by(|a, b| a.name.id.cmp(&b.name.id).then_with(|| a.path.cmp(&b.path)));
        Ok(dirs)
    }

    /// Creates a directory with the next free id: one more than the highest in use.
    pub fn create(&self, label: Option<Label>) -> Result<TrackedDir> {
        // Serialise id allocation with other tempit processes.
        let _lock = self.lock()?;
        let id = match self.list()?.last() {
            Some(dir) => dir.name.id.checked_add(1).ok_or(Error::IdsExhausted)?,
            None => 1,
        };
        let name = DirName { id, label };
        let path = self.root.join(name.to_string());
        fs::create_dir(&path).context("create", &path)?;
        Ok(TrackedDir { name, path })
    }

    /// Finds the directory designated by `reference`, or the most recent one if `None`.
    pub fn resolve(&self, reference: Option<&DirRef>) -> Result<TrackedDir> {
        let dirs = self.list()?;
        match reference {
            Some(reference) => find(dirs, reference),
            None => dirs.into_iter().last().ok_or(Error::Empty),
        }
    }

    fn lock(&self) -> Result<File> {
        let path = self.root.join(".lock");
        let file = File::options()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .context("open", &path)?;
        file.lock().context("lock", &path)?;
        Ok(file)
    }
}

/// Picks the single directory matching `reference`.
fn find(dirs: Vec<TrackedDir>, reference: &DirRef) -> Result<TrackedDir> {
    let mut matches: Vec<TrackedDir> = dirs
        .into_iter()
        .filter(|dir| dir.name.matches(reference))
        .collect();
    match matches.len() {
        0 => Err(Error::NotFound(reference.clone())),
        1 => Ok(matches.remove(0)),
        _ => Err(Error::Ambiguous {
            reference: reference.clone(),
            names: matches.iter().map(|dir| dir.name.to_string()).collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::{PermissionsExt, symlink};

    use super::*;

    fn store() -> (tempfile::TempDir, Store) {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().join("root")).unwrap();
        (tmp, store)
    }

    fn label(s: &str) -> Option<Label> {
        s.parse().ok()
    }

    fn names(store: &Store) -> Vec<String> {
        store
            .list()
            .unwrap()
            .iter()
            .map(|d| d.name.to_string())
            .collect()
    }

    #[test]
    fn open_creates_a_private_root() {
        let (tmp, _store) = store();
        let mode = fs::metadata(tmp.path().join("root"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o700);
    }

    #[test]
    fn open_rejects_a_root_owned_by_someone_else() {
        let tmp = tempfile::tempdir().unwrap();
        let foreign = if rustix::process::getuid().is_root() {
            let dir = tmp.path().join("foreign");
            fs::create_dir(&dir).unwrap();
            std::os::unix::fs::chown(&dir, Some(65_534), None).unwrap();
            dir
        } else {
            PathBuf::from("/")
        };
        assert!(matches!(Store::open(&foreign), Err(Error::ForeignRoot(_))));
    }

    #[test]
    fn open_rejects_a_file() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("file");
        fs::write(&file, "").unwrap();
        assert!(Store::open(&file).is_err());
    }

    #[test]
    fn ids_increase_and_stay_stable() {
        let (_tmp, store) = store();
        store.create(None).unwrap();
        let second = store.create(label("bugfix")).unwrap();
        store.create(None).unwrap();
        assert_eq!(names(&store), ["1", "2-bugfix", "3"]);

        // Removing a directory never renumbers the others.
        second.remove().unwrap();
        assert_eq!(names(&store), ["1", "3"]);

        // The next id follows the highest one.
        store.create(label("next")).unwrap();
        assert_eq!(names(&store), ["1", "3", "4-next"]);
    }

    #[test]
    fn ids_restart_once_everything_is_removed() {
        let (_tmp, store) = store();
        for dir in [store.create(None).unwrap(), store.create(None).unwrap()] {
            dir.remove().unwrap();
        }
        assert_eq!(store.create(None).unwrap().name.id, 1);
    }

    #[test]
    fn create_waits_for_other_processes_to_release_the_lock() {
        let (_tmp, store) = store();
        let held = store.lock().unwrap();
        std::thread::scope(|scope| {
            let create = scope.spawn(|| store.create(None));
            std::thread::sleep(std::time::Duration::from_millis(200));
            assert!(!create.is_finished(), "create must wait for the lock");
            drop(held);
            assert_eq!(create.join().unwrap().unwrap().name.id, 1);
        });
    }

    #[test]
    fn list_ignores_foreign_entries() {
        let (tmp, store) = store();
        let root = tmp.path().join("root");
        store.create(None).unwrap();
        fs::create_dir(root.join("not-ours")).unwrap();
        fs::create_dir(root.join("07")).unwrap();
        fs::write(root.join("8"), "a file").unwrap();
        symlink("/", root.join("9")).unwrap();
        assert_eq!(names(&store), ["1"]);
    }

    #[test]
    fn resolve_by_id_label_or_latest() {
        let (_tmp, store) = store();
        let first = store.create(label("alpha")).unwrap();
        let second = store.create(None).unwrap();

        assert_eq!(store.resolve(Some(&DirRef::Id(1))).unwrap(), first);
        assert_eq!(
            store.resolve(Some(&"alpha".parse().unwrap())).unwrap(),
            first
        );
        assert_eq!(store.resolve(None).unwrap(), second);
        assert!(matches!(
            store.resolve(Some(&DirRef::Id(9))),
            Err(Error::NotFound(DirRef::Id(9)))
        ));
    }

    #[test]
    fn resolve_reports_ambiguous_labels() {
        let (_tmp, store) = store();
        store.create(label("same")).unwrap();
        store.create(label("same")).unwrap();
        match store.resolve(Some(&"same".parse().unwrap())) {
            Err(Error::Ambiguous { names, .. }) => assert_eq!(names, ["1-same", "2-same"]),
            other => panic!("expected an ambiguity error, got {other:?}"),
        }
    }

    #[test]
    fn resolve_latest_of_nothing_is_an_error() {
        let (_tmp, store) = store();
        assert!(matches!(store.resolve(None), Err(Error::Empty)));
    }

    #[test]
    fn save_moves_into_an_existing_directory_under_its_label() {
        let (tmp, store) = store();
        let dir = store.create(label("keep")).unwrap();
        fs::write(dir.path.join("notes.txt"), "important").unwrap();
        let saved_dir = tmp.path().join("saved");
        fs::create_dir(&saved_dir).unwrap();

        let target = dir.save_to(&saved_dir).unwrap();

        assert_eq!(target, saved_dir.join("keep"));
        assert_eq!(
            fs::read_to_string(target.join("notes.txt")).unwrap(),
            "important"
        );
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn save_names_unlabelled_directories_after_their_id() {
        let (tmp, store) = store();
        let dir = store.create(None).unwrap();
        let target = dir.save_to(tmp.path()).unwrap();
        assert_eq!(target, tmp.path().join("tempit-1"));
    }

    #[test]
    fn save_to_a_new_path_renames() {
        let (tmp, store) = store();
        let dir = store.create(None).unwrap();
        let target = dir.save_to(&tmp.path().join("project")).unwrap();
        assert_eq!(target, tmp.path().join("project"));
        assert!(target.is_dir());
    }

    #[test]
    fn save_never_overwrites() {
        let (tmp, store) = store();
        let dir = store.create(label("keep")).unwrap();
        fs::create_dir(tmp.path().join("keep")).unwrap();
        fs::write(tmp.path().join("keep/existing.txt"), "").unwrap();

        assert!(matches!(
            dir.save_to(tmp.path()),
            Err(Error::DestinationExists(_))
        ));
        assert!(dir.path.is_dir());
        assert!(tmp.path().join("keep/existing.txt").exists());
    }

    #[test]
    fn save_refuses_to_move_a_directory_inside_itself() {
        let (_tmp, store) = store();
        let dir = store.create(None).unwrap();
        fs::create_dir(dir.path.join("sub")).unwrap();
        assert!(matches!(
            dir.save_to(&dir.path.join("sub")),
            Err(Error::SaveIntoItself(_))
        ));
    }
}
