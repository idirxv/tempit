//! Command-line interface definition.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::name::{DirRef, Label};
use crate::shell::Shell;

const AFTER_HELP: &str = "\
Directories are referred to by id (e.g. 3) or by label (e.g. bugfix).

Shell integration adds tempc, tempg, templ, temprm, tempsave and tempclean,
which cd for you, plus tab completion. Add this to ~/.bashrc or ~/.zshrc:
  eval \"$(tempit init bash)\"   # or zsh

Environment:
  TEMPIT_ROOT      where directories are created [default: $TMPDIR/tempit-<uid>]
  TEMPIT_SAVE_DIR  where `tempit save` moves directories [default: ~/tempit]";

/// Create, track, jump into and save temporary directories.
#[derive(Debug, Parser)]
#[command(version, after_help = AFTER_HELP)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a temporary directory and print its path
    #[command(visible_alias = "new")]
    Create {
        /// Label appended to the directory name, e.g. `3-bugfix`
        label: Option<Label>,
    },

    /// List tracked directories
    #[command(visible_alias = "ls")]
    List,

    /// Print the path of a directory
    Path {
        /// Id or label of the directory [default: the most recent]
        #[arg(value_name = "REF")]
        reference: Option<DirRef>,
    },

    /// Delete directories and their contents
    #[command(visible_alias = "rm")]
    Remove {
        /// Ids or labels of the directories
        #[arg(value_name = "REF", required = true)]
        references: Vec<DirRef>,
    },

    /// Move a directory out of the temporary area to keep it
    Save {
        /// Id or label of the directory
        #[arg(value_name = "REF")]
        reference: DirRef,

        /// Existing directory to move it into, or its new path [default: the save directory]
        #[arg(value_name = "DEST")]
        destination: Option<PathBuf>,
    },

    /// Delete all tracked directories
    #[command(alias = "clean-all")]
    Clean {
        /// Do not ask for confirmation
        #[arg(short, long)]
        yes: bool,
    },

    /// Print the shell integration script
    Init { shell: Shell },

    /// Print `<id>\t<label>` lines for shell completion
    #[command(name = "__refs", hide = true)]
    Refs,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_definition_is_consistent() {
        Cli::command().debug_assert();
    }
}
