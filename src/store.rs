//! The set of tracked directories. The filesystem is the source of truth: a directory is
//! tracked if and only if it lives in the root and is named `<id>` or `<id>-<label>`.

use std::fs::{self, DirBuilder, File};
use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Component, Path, PathBuf};

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
            let parent = fs::canonicalize(parent).map_err(|err| match err.kind() {
                io::ErrorKind::NotFound => Error::MissingParent(parent.to_path_buf()),
                _ => Error::Io {
                    action: "resolve",
                    path: parent.to_path_buf(),
                    source: err,
                },
            })?;
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

    pub fn root(&self) -> &Path {
        &self.root
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
    ///
    /// Labels are unique, so that a label always designates a single directory.
    pub fn create(&self, label: Option<Label>) -> Result<TrackedDir> {
        // Serialise id allocation and label checks with other tempit processes.
        let _lock = self.lock()?;
        let dirs = self.list()?;
        if let Some(label) = &label
            && let Some(taken) = dirs
                .iter()
                .find(|dir| dir.name.label.as_ref() == Some(label))
        {
            return Err(Error::LabelTaken {
                label: label.clone(),
                name: taken.name.to_string(),
            });
        }
        let id = match dirs.last() {
            Some(dir) => dir.name.id.checked_add(1).ok_or(Error::IdsExhausted)?,
            None => 1,
        };
        let name = DirName { id, label };
        let path = self.root.join(name.to_string());
        fs::create_dir(&path).context("create", &path)?;
        Ok(TrackedDir { name, path })
    }

    /// Finds the directory designated by `reference`. `cwd` is what `.` is relative to.
    pub fn resolve(&self, reference: &DirRef, cwd: Option<&Path>) -> Result<TrackedDir> {
        if *reference == DirRef::Current {
            let current = match cwd {
                Some(cwd) => self.containing(cwd)?,
                None => None,
            };
            return current.ok_or(Error::NotInTempDir);
        }
        find(&self.list()?, reference)
    }

    /// The most recently created directory.
    pub fn latest(&self) -> Result<TrackedDir> {
        self.list()?.pop().ok_or(Error::Empty)
    }

    /// The tracked directory that is or contains `path`, if any.
    pub fn containing(&self, path: &Path) -> Result<Option<TrackedDir>> {
        let root = fs::canonicalize(&self.root).context("resolve", &self.root)?;
        // A path that cannot be resolved, e.g. because it was deleted, is in no directory.
        let Ok(path) = fs::canonicalize(path) else {
            return Ok(None);
        };
        let first = path
            .strip_prefix(&root)
            .ok()
            .and_then(|inside| inside.components().next());
        let Some(Component::Normal(name)) = first else {
            return Ok(None);
        };
        Ok(self
            .list()?
            .into_iter()
            .find(|dir| dir.path.file_name() == Some(name)))
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

/// Picks the single directory matching `reference` by id or label.
fn find(dirs: &[TrackedDir], reference: &DirRef) -> Result<TrackedDir> {
    let matches: Vec<&TrackedDir> = dirs
        .iter()
        .filter(|dir| dir.name.matches(reference))
        .collect();
    match matches.as_slice() {
        [dir] => Ok((*dir).clone()),
        [] => Err(Error::NotFound {
            reference: reference.clone(),
            similar: similar_label(dirs, reference),
        }),
        _ => Err(Error::Ambiguous {
            reference: reference.clone(),
            names: matches.iter().map(|dir| dir.name.to_string()).collect(),
        }),
    }
}

/// The existing label closest to a mistyped one, if one is close enough: a label the input
/// is a prefix of, or one within a few typos (Damerau-Levenshtein, which counts a
/// transposition as one edit; case is ignored).
fn similar_label(dirs: &[TrackedDir], reference: &DirRef) -> Option<Label> {
    let DirRef::Label(wanted) = reference else {
        return None;
    };
    let wanted = wanted.as_str().to_lowercase();
    let max_typos = (wanted.chars().count() / 3).max(1);
    dirs.iter()
        .filter_map(|dir| dir.name.label.as_ref())
        .map(|label| {
            let candidate = label.as_str().to_lowercase();
            let distance = if candidate.starts_with(&wanted) {
                0
            } else {
                strsim::damerau_levenshtein(&wanted, &candidate)
            };
            (distance, label)
        })
        .filter(|(distance, _)| *distance <= max_typos)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, label)| label.clone())
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
    fn labels_are_unique() {
        let (_tmp, store) = store();
        store.create(label("api")).unwrap();
        match store.create(label("api")) {
            Err(Error::LabelTaken { name, .. }) => assert_eq!(name, "1-api"),
            other => panic!("expected the label to be taken, got {other:?}"),
        }
        assert_eq!(names(&store), ["1-api"]);
    }

    #[test]
    fn resolve_by_id_or_label() {
        let (_tmp, store) = store();
        let first = store.create(label("alpha")).unwrap();
        let second = store.create(None).unwrap();

        assert_eq!(store.resolve(&DirRef::Id(1), None).unwrap(), first);
        assert_eq!(
            store.resolve(&"alpha".parse().unwrap(), None).unwrap(),
            first
        );
        assert_eq!(store.latest().unwrap(), second);
        assert!(matches!(
            store.resolve(&DirRef::Id(9), None),
            Err(Error::NotFound {
                reference: DirRef::Id(9),
                similar: None
            })
        ));
    }

    #[test]
    fn resolve_suggests_a_similar_label() {
        let (_tmp, store) = store();
        store.create(label("test")).unwrap();
        store.create(label("api")).unwrap();
        store.create(label("bugfix")).unwrap();

        let similar = |wanted: &str| match store.resolve(&wanted.parse().unwrap(), None) {
            Err(Error::NotFound { similar, .. }) => similar.map(|l| l.to_string()),
            other => panic!("expected not found, got {other:?}"),
        };
        assert_eq!(similar("tst").as_deref(), Some("test"));
        assert_eq!(similar("apo").as_deref(), Some("api"));
        assert_eq!(similar("aip").as_deref(), Some("api"), "transposition");
        assert_eq!(similar("API").as_deref(), Some("api"), "case");
        assert_eq!(similar("bu").as_deref(), Some("bugfix"), "prefix");
        assert_eq!(similar("bugfxi").as_deref(), Some("bugfix"));
        assert_eq!(similar("zzz"), None);
        assert_eq!(similar("xyz"), None);
    }

    #[test]
    fn resolve_the_current_directory_from_anywhere_inside_it() {
        let (tmp, store) = store();
        store.create(None).unwrap();
        let dir = store.create(label("work")).unwrap();
        let nested = dir.path.join("src/deep");
        fs::create_dir_all(&nested).unwrap();

        for cwd in [&dir.path, &nested] {
            assert_eq!(store.resolve(&DirRef::Current, Some(cwd)).unwrap(), dir);
        }
        for cwd in [Some(tmp.path()), Some(store.root()), None] {
            assert!(matches!(
                store.resolve(&DirRef::Current, cwd),
                Err(Error::NotInTempDir)
            ));
        }
    }

    #[test]
    fn containing_ignores_paths_that_do_not_exist() {
        let (_tmp, store) = store();
        let dir = store.create(None).unwrap();
        assert_eq!(store.containing(&dir.path.join("missing")).unwrap(), None);
    }

    #[test]
    fn resolve_reports_ambiguous_labels() {
        // Only possible with directories created by hand: `create` keeps labels unique.
        let (_tmp, store) = store();
        fs::create_dir(store.root().join("1-same")).unwrap();
        fs::create_dir(store.root().join("2-same")).unwrap();
        match store.resolve(&"same".parse().unwrap(), None) {
            Err(Error::Ambiguous { names, .. }) => assert_eq!(names, ["1-same", "2-same"]),
            other => panic!("expected an ambiguity error, got {other:?}"),
        }
    }

    #[test]
    fn latest_of_nothing_is_an_error() {
        let (_tmp, store) = store();
        assert!(matches!(store.latest(), Err(Error::Empty)));
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
    fn save_explains_a_missing_parent() {
        let (tmp, store) = store();
        let dir = store.create(None).unwrap();
        let missing = tmp.path().join("missing");
        assert!(matches!(
            dir.save_to(&missing.join("project")),
            Err(Error::MissingParent(p)) if p == missing
        ));
        assert!(dir.path.is_dir());
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
