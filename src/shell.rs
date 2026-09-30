//! Shell integration scripts, embedded in the binary.

use clap::ValueEnum;

use crate::error::{Error, Result};

const BASH: &str = concat!(
    include_str!("../shell/common.sh"),
    include_str!("../shell/completion.bash")
);
const ZSH: &str = concat!(
    include_str!("../shell/common.sh"),
    include_str!("../shell/completion.zsh")
);

/// Shells tempit can integrate with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
}

impl Shell {
    /// The shell named `name` (typically the basename of `$SHELL`).
    pub fn detect(name: Option<&str>) -> Result<Self> {
        let name = name.ok_or(Error::UnknownShell(None))?;
        Self::from_str(name, true).map_err(|_| Error::UnknownShell(Some(name.to_owned())))
    }

    /// Script defining the `temp*` functions and tab completion, meant to be `eval`ed.
    pub fn init_script(self) -> &'static str {
        match self {
            Self::Bash => BASH,
            Self::Zsh => ZSH,
        }
    }

    /// What to tell a person who runs `tempit init` in a terminal rather than `eval`ing it.
    pub fn setup_instructions(self) -> String {
        let name = self.name();
        let where_ = match self {
            Self::Bash => "~/.bashrc",
            Self::Zsh => "~/.zshrc, after compinit",
        };
        format!(
            "To enable the tempc, tempg, templ, temprm, tempsave and tempclean functions and tab\n\
             completion, add this line to {where_}:\n\
             \n    eval \"$(tempit init {name})\"\n\
             \nthen open a new terminal. To read the script itself: tempit init {name} | less\n"
        )
    }

    fn name(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;
    use crate::cli::Cli;

    /// Completion scripts list subcommands by hand: make sure none is forgotten.
    #[test]
    fn completion_scripts_know_every_subcommand() {
        let cli = Cli::command();
        for shell in Shell::value_variants() {
            let script = shell.init_script();
            for sub in cli.get_subcommands().filter(|sub| !sub.is_hide_set()) {
                let names = std::iter::once(sub.get_name()).chain(sub.get_visible_aliases());
                for name in names {
                    assert!(
                        script.contains(name),
                        "{shell:?} completion does not mention `{name}`"
                    );
                }
            }
        }
    }

    #[test]
    fn detects_the_shell_by_name() {
        assert_eq!(Shell::detect(Some("zsh")).unwrap(), Shell::Zsh);
        assert_eq!(Shell::detect(Some("bash")).unwrap(), Shell::Bash);
        assert!(matches!(
            Shell::detect(Some("fish")),
            Err(Error::UnknownShell(Some(name))) if name == "fish"
        ));
        assert!(matches!(
            Shell::detect(None),
            Err(Error::UnknownShell(None))
        ));
    }

    #[test]
    fn instructions_and_tips_cover_every_shell() {
        let tip = Error::UnknownShell(None).tip().unwrap();
        for shell in Shell::value_variants() {
            let instructions = shell.setup_instructions();
            let line = format!("eval \"$(tempit init {})\"", shell.name());
            assert!(instructions.contains(&line), "{instructions}");
            assert!(
                tip.contains(shell.name()),
                "the tip does not mention {shell:?}"
            );
        }
    }

    #[test]
    fn completion_scripts_know_every_shell() {
        for shell in Shell::value_variants() {
            let name = shell.to_possible_value().unwrap();
            for script in [BASH, ZSH] {
                assert!(script.contains(name.get_name()));
            }
        }
    }
}
