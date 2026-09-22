//! Error type shared by every command.

use std::io;
use std::path::{Path, PathBuf};

use crate::name::DirRef;

/// Result alias using [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything that can make a tempit command fail.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no tracked directory matches {0}")]
    NotFound(DirRef),

    #[error("{reference} matches several directories ({}); use its id instead", .names.join(", "))]
    Ambiguous {
        reference: DirRef,
        names: Vec<String>,
    },

    #[error("there are no tracked directories")]
    Empty,

    #[error("ids are exhausted; clean up some directories first")]
    IdsExhausted,

    #[error(
        "{} is owned by another user; refusing to use it (set TEMPIT_ROOT to pick another location)",
        .0.display()
    )]
    ForeignRoot(PathBuf),

    #[error("{} is not a directory", .0.display())]
    NotADirectory(PathBuf),

    #[error("{} already exists", .0.display())]
    DestinationExists(PathBuf),

    #[error("cannot save {} inside itself", .0.display())]
    SaveIntoItself(PathBuf),

    #[error("cannot copy {}: only files, directories and symlinks are supported", .0.display())]
    UnsupportedFileType(PathBuf),

    #[error("cannot find a default save directory; set TEMPIT_SAVE_DIR or pass a destination")]
    NoSaveDir,

    #[error("refusing to delete without confirmation; pass --yes")]
    ConfirmationRequired,

    #[error("aborted")]
    Aborted,

    #[error("cannot {action} {}", .path.display())]
    Io {
        action: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("cannot write output")]
    Output(#[source] io::Error),

    #[error("cannot read input")]
    Input(#[source] io::Error),
}

/// Attaches the failed action and the path involved to an I/O error.
pub trait IoContext<T> {
    fn context(self, action: &'static str, path: &Path) -> Result<T>;
}

impl<T> IoContext<T> for io::Result<T> {
    fn context(self, action: &'static str, path: &Path) -> Result<T> {
        self.map_err(|source| Error::Io {
            action,
            path: path.to_path_buf(),
            source,
        })
    }
}
