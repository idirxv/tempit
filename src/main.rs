//! CLI entry point for `tempit`.

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use tempit::TempitManager;

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
    /// Initialize Tempit in the current shell (bash, zsh)
    Init {
        /// Shell name (e.g., bash, zsh)
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
    /// Get directory path by number
    #[command(hide = true)]
    Path {
        /// Number of the directory
        #[arg(allow_hyphen_values = true)]
        number: String,
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

fn parse_number_arg(subcmd: &str, raw: &str) -> Result<Option<i64>, ExitCode> {
    if raw.starts_with('-') && raw.len() >= 2 {
        return Err(print_no_such_option(subcmd, raw));
    }
    if !raw.is_empty() && raw.bytes().all(|b| b.is_ascii_digit()) {
        match raw.parse::<i64>() {
            Ok(n) => Ok(Some(n)),
            Err(_) => {
                eprintln!("ERROR: Invalid directory number: {raw}");
                Ok(None)
            }
        }
    } else {
        eprintln!("ERROR: Invalid value for 'NUMBER': '{raw}' is not a valid integer.");
        Err(ExitCode::from(2))
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
        Commands::Path { number } => {
            let n = match parse_number_arg("path", &number) {
                Ok(Some(n)) => n,
                Ok(None) => {
                    println!();
                    return ExitCode::SUCCESS;
                }
                Err(code) => return code,
            };
            match mgr.get_path_by_number(n) {
                Some(path) => println!("{}", path.display()),
                None => println!(),
            }
            ExitCode::SUCCESS
        }
    }
}
