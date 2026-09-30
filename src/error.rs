//! Error type shared by every command.

use std::io;
use std::path::{Path, PathBuf};

use crate::name::{DirRef, Label};

/// Result alias using [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything that can make a tempit command fail.
///
/// Messages say what went wrong; [`Error::tip`] says what to do about it.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no directory matches {reference}")]
    NotFound {
        reference: DirRef,
        /// An existing label close to the one asked for.
        similar: Option<Label>,
    },

    #[error("{reference} matches several directories: {}", .names.join(", "))]
    Ambiguous {
        reference: DirRef,
        names: Vec<String>,
    },

    #[error("there are no temporary directories yet")]
    Empty,

    #[error("the current directory is not inside a temporary directory")]
    NotInTempDir,

    #[error("label '{label}' is already used by {name}")]
    LabelTaken { label: Label, name: String },

    #[error("ids are exhausted")]
    IdsExhausted,

    #[error("{} belongs to another user", .0.display())]
    ForeignRoot(PathBuf),

    #[error("{} is not a directory", .0.display())]
    NotADirectory(PathBuf),

    #[error("{} already exists", .0.display())]
    DestinationExists(PathBuf),

    #[error("{} does not exist", .0.display())]
    MissingParent(PathBuf),

    #[error("cannot save {} inside itself", .0.display())]
    SaveIntoItself(PathBuf),

    #[error("cannot copy {}: only files, directories and symlinks are supported", .0.display())]
    UnsupportedFileType(PathBuf),

    #[error("cannot find a default save directory")]
    NoSaveDir,

    #[error("refusing to delete {0} without confirmation")]
    ConfirmationRequired(String),

    #[error("{}", match .0 {
        Some(shell) => format!("unsupported shell '{shell}'"),
        None => "cannot detect your shell: $SHELL is not set".to_owned(),
    })]
    UnknownShell(Option<String>),

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

impl Error {
    /// A suggestion of what to do next, shown under the error message.
    pub fn tip(&self) -> Option<String> {
        let tip = match self {
            Self::NotFound {
                similar: Some(label),
                ..
            } => format!("did you mean '{label}'?"),
            Self::NotFound { .. } | Self::NotInTempDir => {
                "`tempit list` shows the ids and labels in use".to_owned()
            }
            Self::Ambiguous { .. } => "refer to the directory by its id instead".to_owned(),
            Self::Empty => "create one with `tempit create [LABEL]`".to_owned(),
            Self::LabelTaken { label, .. } => {
                format!("pick another label, or go there with `tempg {label}`")
            }
            Self::IdsExhausted => "delete some directories first".to_owned(),
            Self::ForeignRoot(_) => "set TEMPIT_ROOT to use another location".to_owned(),
            Self::DestinationExists(_) => "pick another destination".to_owned(),
            Self::MissingParent(_) => "create it first, or save somewhere else".to_owned(),
            Self::NoSaveDir => "set TEMPIT_SAVE_DIR, or pass a destination".to_owned(),
            Self::ConfirmationRequired(_) => "pass --yes to confirm".to_owned(),
            Self::UnknownShell(_) => {
                "tempit supports bash and zsh: `tempit init bash` or `tempit init zsh`".to_owned()
            }
            _ => return None,
        };
        Some(tip)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_suggests_a_similar_label_or_the_list() {
        let reference = DirRef::Label("tst".parse().unwrap());
        let with_similar = Error::NotFound {
            reference: reference.clone(),
            similar: Some("test".parse().unwrap()),
        };
        assert_eq!(with_similar.to_string(), "no directory matches label 'tst'");
        assert_eq!(with_similar.tip().unwrap(), "did you mean 'test'?");

        let without = Error::NotFound {
            reference,
            similar: None,
        };
        assert!(without.tip().unwrap().contains("tempit list"));
    }

    #[test]
    fn unknown_shell_explains_what_is_supported() {
        let err = Error::UnknownShell(Some("fish".to_owned()));
        assert_eq!(err.to_string(), "unsupported shell 'fish'");
        assert!(err.tip().unwrap().contains("bash and zsh"));
        assert!(Error::UnknownShell(None).to_string().contains("$SHELL"));
    }
}
