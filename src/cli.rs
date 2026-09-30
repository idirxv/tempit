//! Command-line interface definition.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::name::{DirRef, Label};
use crate::shell::Shell;

const AFTER_HELP: &str = "\
Without a command, tempit lists your directories. Refer to a directory by its
id (3), its label (bugfix), or `.` for the one you are in.

Shell integration adds tempc, tempg, templ, temprm, tempsave and tempclean,
which cd for you, plus tab completion. Run `tempit init` to set it up.

Environment:
  TEMPIT_ROOT      where directories are created [default: $TMPDIR/tempit-<uid>]
  TEMPIT_SAVE_DIR  where `tempit save` moves directories [default: ~/tempit]";

/// Create, track, jump into and save temporary directories.
#[derive(Debug, Parser)]
#[command(version, after_help = AFTER_HELP)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a temporary directory and print its path
    #[command(visible_alias = "new")]
    Create {
        /// Unique label appended to the directory name, e.g. `3-bugfix`
        label: Option<Label>,
    },

    /// List directories (the default command)
    #[command(visible_alias = "ls")]
    List,

    /// Print the path of a directory
    Path {
        /// Id, label or `.` [default: the most recent directory]
        #[arg(value_name = "REF")]
        reference: Option<DirRef>,
    },

    /// Delete directories and their contents
    #[command(visible_alias = "rm")]
    Remove {
        /// Ids, labels or `.`
        #[arg(value_name = "REF", required = true)]
        references: Vec<DirRef>,
    },

    /// Move a directory out of the temporary area to keep it
    Save {
        /// Id, label or `.` [default: the directory you are in]
        #[arg(value_name = "REF")]
        reference: Option<DirRef>,

        /// Existing directory to move it into, or its new path [default: the save directory]
        #[arg(value_name = "DEST")]
        destination: Option<PathBuf>,
    },

    /// Delete all directories, after confirmation
    #[command(alias = "clean-all")]
    Clean {
        /// Do not ask for confirmation
        #[arg(short, long)]
        yes: bool,
    },

    /// Set up the shell integration (prints the script to `eval`)
    Init {
        /// Shell to integrate with [default: detected from $SHELL]
        shell: Option<Shell>,
    },

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
