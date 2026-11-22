//! CLI entry point for `tempit`.

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use tempit::{PrefixLookup, TempitManager};

#[derive(Parser)]
#[command(
    name = "tempit",
    about = "Manage and jump into temporary working directories",
    disable_version_flag = true
)]
struct Cli {
    /// Show version and exit
    #[arg(short = 'v', long = "version")]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new temporary directory
    Create {
        /// Prefix for the temporary directory
        #[arg(default_value = "tempit")]
        name: String,
    },
    /// Initialize Tempit in the current shell (bash, zsh, fish)
    Init {
        /// Shell name (e.g., bash, zsh, fish)
        shell: String,
    },
    /// List all tracked temporary directories
    List,
    /// Remove a tracked temporary directory by its number
    Remove {
        /// Number of the directory to remove
        #[arg(allow_hyphen_values = true)]
        number: String,
    },
    /// Remove all tracked temporary directories
    CleanAll,
    /// Get directory path by number or prefix, or the most recent directory if omitted
    #[command(hide = true)]
    Path {
        /// Number or prefix of the directory (defaults to most recently created)
        #[arg(allow_hyphen_values = true)]
        target: Option<String>,
    },
}

fn get_manager() -> Result<TempitManager, ExitCode> {
    TempitManager::from_defaults().map_err(|e| {
        eprintln!("ERROR: An error occurred: {e}");
        ExitCode::from(1)
    })
}

fn print_no_such_option(subcmd: &str, arg: &str) -> ExitCode {
    let opt: String = arg.chars().take(2).collect();
    let msg = format!("No such option: {opt}");
    let width = 76usize;
    let padded = format!("{msg:<width$}");
    eprintln!("Usage: tempit {subcmd} [OPTIONS] NUMBER");
    eprintln!("Try 'tempit {subcmd} --help' for help.");
    eprintln!("╭─ Error ──────────────────────────────────────────────────────────────────────╮");
    eprintln!("│ {padded} │");
    eprintln!("╰──────────────────────────────────────────────────────────────────────────────╯");
    ExitCode::from(2)
}

enum RawArg {
    Option,
    Number(i64),
    Overflow,
    NonNumeric,
}

fn classify_arg(raw: &str) -> RawArg {
    if raw.starts_with('-') && raw.len() >= 2 {
        return RawArg::Option;
    }
    if !raw.is_empty() && raw.bytes().all(|b| b.is_ascii_digit()) {
        return match raw.parse::<i64>() {
            Ok(n) => RawArg::Number(n),
            Err(_) => RawArg::Overflow,
        };
    }
    RawArg::NonNumeric
}

fn parse_number_arg(subcmd: &str, raw: &str) -> Result<Option<i64>, ExitCode> {
    match classify_arg(raw) {
        RawArg::Option => Err(print_no_such_option(subcmd, raw)),
        RawArg::Number(n) => Ok(Some(n)),
        RawArg::Overflow => {
            eprintln!("ERROR: Invalid directory number: {raw}");
            Ok(None)
        }
        RawArg::NonNumeric => {
            eprintln!("ERROR: Invalid value for 'NUMBER': '{raw}' is not a valid integer.");
            Err(ExitCode::from(2))
        }
    }
}

fn resolve_path_arg(mgr: &TempitManager, raw: &str) -> ExitCode {
    match classify_arg(raw) {
        RawArg::Option => return print_no_such_option("path", raw),
        RawArg::Overflow => {
            eprintln!("ERROR: Invalid directory number: {raw}");
            println!();
            return ExitCode::SUCCESS;
        }
        RawArg::Number(n) => {
            match mgr.get_path_by_number(n) {
                Some(path) => println!("{}", path.display()),
                None => println!(),
            }
            return ExitCode::SUCCESS;
        }
        RawArg::NonNumeric => {}
    }

    match mgr.get_path_by_prefix(raw) {
        PrefixLookup::Matched(path) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        PrefixLookup::NotFound => {
            eprintln!("ERROR: No temporary directory matching prefix '{raw}'.");
            ExitCode::from(1)
        }
        PrefixLookup::Ambiguous(matches) => {
            eprintln!("ERROR: Ambiguous prefix '{raw}'. Matching directories:");
            for candidate in matches {
                eprintln!(
                    "  {}: {} ({})",
                    candidate.index,
                    candidate.info.prefix,
                    candidate.info.path.display()
                );
            }
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    let Some(command) = cli.command else {
        let _ = <Cli as clap::CommandFactory>::command().print_help();
        println!();
        return ExitCode::SUCCESS;
    };

    let mgr = match get_manager() {
        Ok(m) => m,
        Err(code) => return code,
    };

    match command {
        Commands::Create { name } => match mgr.create(&name) {
            Ok(path) => {
                println!("{}", path.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("ERROR: Error creating temporary directory: {e}");
                ExitCode::from(1)
            }
        },
        Commands::Init { shell } => {
            mgr.init_shell(&shell);
            ExitCode::SUCCESS
        }
        Commands::List => match mgr.print_directories() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("ERROR: {e}");
                ExitCode::from(1)
            }
        },
        Commands::CleanAll => match mgr.clean_all_directories() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("ERROR: {e}");
                ExitCode::from(1)
            }
        },
        Commands::Remove { number } => {
            let n = match parse_number_arg("remove", &number) {
                Ok(Some(n)) => n,
                Ok(None) => return ExitCode::from(1),
                Err(code) => return code,
            };
            if mgr.remove(n) {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Commands::Path { target } => match target {
            Some(raw) => resolve_path_arg(&mgr, &raw),
            None => match mgr.get_most_recent_path() {
                Some(path) => {
                    println!("{}", path.display());
                    ExitCode::SUCCESS
                }
                None => {
                    eprintln!("ERROR: No temporary directories found.");
                    ExitCode::from(1)
                }
            },
        },
    }
}
