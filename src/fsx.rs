//! Filesystem helpers that `std` lacks: a no-clobber move that works across filesystems.

use std::fs::{self, Metadata};
use std::io;
use std::os::unix::fs::{MetadataExt, symlink};
use std::path::Path;

use rustix::fs::{AtFlags, CWD, Timespec, Timestamps};

use crate::error::{Error, IoContext, Result};

/// Moves the directory `from` to `to`, which must not exist yet.
///
/// Renames when possible. Across filesystems (e.g. from a tmpfs `/tmp` to a disk-backed home),
/// falls back to copying the tree and then deleting the original.
pub fn move_dir(from: &Path, to: &Path) -> Result<()> {
    match rename_no_replace(from, to) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            Err(Error::DestinationExists(to.to_path_buf()))
        }
        Err(err) if err.kind() == io::ErrorKind::CrossesDevices => {
            copy_tree(from, to)?;
            fs::remove_dir_all(from).context("remove", from)
        }
        Err(err) => Err(err).context("move", from),
    }
}

/// Renames atomically, failing with `AlreadyExists` instead of replacing an existing `to`.
fn rename_no_replace(from: &Path, to: &Path) -> io::Result<()> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    match rustix::fs::renameat_with(CWD, from, CWD, to, rustix::fs::RenameFlags::NOREPLACE) {
        // Some filesystems (e.g. NFS) do not support the flag: fall through to check-then-rename.
        Err(rustix::io::Errno::INVAL) => {}
        result => return result.map_err(io::Error::from),
    }
    if to.symlink_metadata().is_ok() {
        return Err(io::ErrorKind::AlreadyExists.into());
    }
    fs::rename(from, to)
}

/// Recursively copies the directory `from` to `to`, which must not exist yet.
///
/// Symlinks are copied as links, and permissions and timestamps are preserved. Any other kind
/// of file (sockets, FIFOs, devices) is an error. On failure, the partial copy is removed.
pub fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    let meta = fs::metadata(from).context("inspect", from)?;
    fs::create_dir(to).map_err(|err| match err.kind() {
        io::ErrorKind::AlreadyExists => Error::DestinationExists(to.to_path_buf()),
        _ => Error::Io {
            action: "create",
            path: to.to_path_buf(),
            source: err,
        },
    })?;
    let result = copy_dir(from, to, &meta);
    if result.is_err() {
        // Best effort: the original error is what matters to the user.
        let _ = fs::remove_dir_all(to);
    }
    result
}

/// Copies the entries of `from` into the existing directory `to`, then gives `to` the
/// permissions and times recorded in `meta`.
fn copy_dir(from: &Path, to: &Path, meta: &Metadata) -> Result<()> {
    for entry in fs::read_dir(from).context("read", from)? {
        let entry = entry.context("read", from)?;
        let (source, target) = (entry.path(), to.join(entry.file_name()));
        // Does not follow symlinks.
        let entry_meta = entry.metadata().context("inspect", &source)?;
        let file_type = entry_meta.file_type();

        if file_type.is_dir() {
            fs::create_dir(&target).context("create", &target)?;
            copy_dir(&source, &target, &entry_meta)?;
        } else if file_type.is_file() {
            fs::copy(&source, &target).context("copy", &source)?;
            copy_times(&entry_meta, &target)?;
        } else if file_type.is_symlink() {
            let link = fs::read_link(&source).context("read", &source)?;
            symlink(&link, &target).context("create", &target)?;
            copy_times(&entry_meta, &target)?;
        } else {
            return Err(Error::UnsupportedFileType(source));
        }
    }
    // Applied last: adding entries changes the times, and a read-only directory could not
    // receive them.
    fs::set_permissions(to, meta.permissions()).context("set permissions on", to)?;
    copy_times(meta, to)
}

/// Gives `target` the access and modification times recorded in `meta`, without following
/// symlinks.
fn copy_times(meta: &Metadata, target: &Path) -> Result<()> {
    // Nanoseconds are always below 10^9, so the cast is lossless on every platform.
    #[allow(clippy::cast_possible_truncation, clippy::unnecessary_cast)]
    let times = Timestamps {
        last_access: Timespec {
            tv_sec: meta.atime(),
            tv_nsec: meta.atime_nsec() as _,
        },
        last_modification: Timespec {
            tv_sec: meta.mtime(),
            tv_nsec: meta.mtime_nsec() as _,
        },
    };
    rustix::fs::utimensat(CWD, target, &times, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(io::Error::from)
        .context("set times on", target)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;
    use std::time::{Duration, SystemTime};

    use super::*;

    fn mode(path: &Path) -> u32 {
        fs::symlink_metadata(path).unwrap().permissions().mode() & 0o7777
    }

    fn mtime(path: &Path) -> SystemTime {
        fs::symlink_metadata(path).unwrap().modified().unwrap()
    }

    /// Builds a small tree with a nested directory, a restricted file and a symlink.
    fn sample_tree(root: &Path) -> SystemTime {
        let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
        fs::create_dir_all(root.join("sub/empty")).unwrap();
        fs::write(root.join("sub/data.txt"), "hello").unwrap();
        fs::set_permissions(root.join("sub/data.txt"), fs::Permissions::from_mode(0o640)).unwrap();
        fs::File::options()
            .write(true)
            .open(root.join("sub/data.txt"))
            .unwrap()
            .set_modified(old)
            .unwrap();
        symlink("sub/data.txt", root.join("link")).unwrap();
        fs::set_permissions(root.join("sub"), fs::Permissions::from_mode(0o750)).unwrap();
        old
    }

    #[test]
    fn copy_tree_preserves_content_modes_times_and_links() {
        let tmp = tempfile::tempdir().unwrap();
        let (from, to) = (tmp.path().join("from"), tmp.path().join("to"));
        fs::create_dir(&from).unwrap();
        let old = sample_tree(&from);

        copy_tree(&from, &to).unwrap();

        assert_eq!(
            fs::read_to_string(to.join("sub/data.txt")).unwrap(),
            "hello"
        );
        assert!(to.join("sub/empty").is_dir());
        assert_eq!(
            fs::read_link(to.join("link")).unwrap(),
            Path::new("sub/data.txt")
        );
        assert_eq!(mode(&to.join("sub/data.txt")), 0o640);
        assert_eq!(mode(&to.join("sub")), 0o750);
        assert_eq!(mtime(&to.join("sub/data.txt")), old);
        assert_eq!(mtime(&to.join("sub")), mtime(&from.join("sub")));
        assert!(
            from.join("sub/data.txt").exists(),
            "copy must not touch the source"
        );
    }

    #[test]
    fn copy_tree_refuses_existing_destination() {
        let tmp = tempfile::tempdir().unwrap();
        let (from, to) = (tmp.path().join("from"), tmp.path().join("to"));
        fs::create_dir(&from).unwrap();
        fs::create_dir(&to).unwrap();

        assert!(matches!(copy_tree(&from, &to), Err(Error::DestinationExists(p)) if p == to));
    }

    #[test]
    fn copy_tree_rolls_back_on_unsupported_files() {
        let tmp = tempfile::tempdir().unwrap();
        let (from, to) = (tmp.path().join("from"), tmp.path().join("to"));
        fs::create_dir(&from).unwrap();
        fs::write(from.join("a.txt"), "a").unwrap();
        let _socket = UnixListener::bind(from.join("server.sock")).unwrap();

        let err = copy_tree(&from, &to).unwrap_err();

        assert!(matches!(err, Error::UnsupportedFileType(p) if p.ends_with("server.sock")));
        assert!(!to.exists(), "the partial copy must be removed");
    }

    #[test]
    fn move_dir_renames_and_never_overwrites() {
        let tmp = tempfile::tempdir().unwrap();
        let (from, to) = (tmp.path().join("from"), tmp.path().join("to"));
        fs::create_dir(&from).unwrap();
        fs::write(from.join("a.txt"), "a").unwrap();
        fs::create_dir(&to).unwrap();

        assert!(matches!(
            move_dir(&from, &to),
            Err(Error::DestinationExists(_))
        ));
        assert!(from.join("a.txt").exists());

        fs::remove_dir(&to).unwrap();
        move_dir(&from, &to).unwrap();
        assert!(!from.exists());
        assert_eq!(fs::read_to_string(to.join("a.txt")).unwrap(), "a");
    }
}
