//! Shell integration scripts, embedded in the binary.

use clap::ValueEnum;

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
    /// Script defining the `temp*` functions and tab completion, meant to be `eval`ed.
    pub fn init_script(self) -> &'static str {
        match self {
            Self::Bash => BASH,
            Self::Zsh => ZSH,
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
    fn completion_scripts_know_every_shell() {
        for shell in Shell::value_variants() {
            let name = shell.to_possible_value().unwrap();
            for script in [BASH, ZSH] {
                assert!(script.contains(name.get_name()));
            }
        }
    }
}
