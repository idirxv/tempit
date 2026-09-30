//! tempit: create, track, jump into and save temporary directories.

#[cfg(not(unix))]
compile_error!("tempit supports Unix-like systems only");

mod cli;
mod config;
mod error;
mod fsx;
mod name;
mod render;
mod shell;
mod stats;
mod store;

use std::error::Error as _;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::SystemTime;
use std::{fs, iter};

use anstyle::{AnsiColor, Style};
use clap::Parser;

use crate::cli::{Cli, Command};
use crate::config::Config;
use crate::error::{Error, IoContext, Result};
use crate::name::DirRef;
use crate::render::{Row, human_size, plural};
use crate::shell::Shell;
use crate::stats::DirStats;
use crate::store::{Store, TrackedDir};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli.command.unwrap_or(Command::List), &Config::from_env()) {
        Ok(()) => ExitCode::SUCCESS,
        // The reader went away (e.g. `tempit list | head -1`): nothing left to say.
        Err(Error::Output(err)) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(err) => {
            report(&err);
            ExitCode::FAILURE
        }
    }
}

fn run(command: Command, config: &Config) -> Result<()> {
    let store = || Store::open(&config.root);
    let cwd = config.cwd.as_deref();
    match command {
        Command::Create { label } => print_path(&store()?.create(label)?.path),
        Command::List => list(&store()?, cwd),
        Command::Path { reference } => {
            let store = store()?;
            let dir = match reference {
                Some(reference) => store.resolve(&reference, cwd)?,
                None => store.latest()?,
            };
            print_path(&dir.path)
        }
        Command::Remove { references } => remove(&store()?, &references, cwd),
        Command::Save {
            reference,
            destination,
        } => {
            let reference = reference.unwrap_or(DirRef::Current);
            save(&store()?, &reference, destination, config)
        }
        Command::Clean { yes } => clean(&store()?, yes),
        Command::Init { shell } => init(shell, config),
        Command::Refs => refs(&store()?),
    }
}

fn list(store: &Store, cwd: Option<&Path>) -> Result<()> {
    let dirs = store.list()?;
    if dirs.is_empty() {
        eprintln!(
            "No temporary directories yet. Create one with `tempit create [LABEL]`, \
             or see `tempit --help`."
        );
        return Ok(());
    }
    let current = match cwd {
        Some(cwd) => store.containing(cwd)?,
        None => None,
    };
    let rows: Vec<Row> = dirs
        .into_iter()
        .map(|dir| Row {
            current: current.as_ref() == Some(&dir),
            stats: DirStats::collect(&dir.path),
            dir,
        })
        .collect();
    let mut out = anstream::stdout().lock();
    render::write_table(&mut out, &rows, store.root(), SystemTime::now()).map_err(Error::Output)
}

fn remove(store: &Store, references: &[DirRef], cwd: Option<&Path>) -> Result<()> {
    // Resolve everything first, so that a typo does not leave the job half done.
    let mut dirs: Vec<TrackedDir> = Vec::new();
    for reference in references {
        let dir = store.resolve(reference, cwd)?;
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    for dir in dirs {
        dir.remove()?;
        eprintln!("Removed {}", dir.name);
    }
    Ok(())
}

fn save(
    store: &Store,
    reference: &DirRef,
    destination: Option<PathBuf>,
    config: &Config,
) -> Result<()> {
    let dir = store.resolve(reference, config.cwd.as_deref())?;
    let destination = if let Some(destination) = destination {
        config::absolute(destination)
    } else {
        let save_dir = config.save_dir.clone().ok_or(Error::NoSaveDir)?;
        fs::create_dir_all(&save_dir).context("create", &save_dir)?;
        save_dir
    };
    print_path(&dir.save_to(&destination)?)
}

fn clean(store: &Store, yes: bool) -> Result<()> {
    let dirs = store.list()?;
    if dirs.is_empty() {
        eprintln!("Nothing to clean.");
        return Ok(());
    }
    let sizes: Vec<u64> = dirs
        .iter()
        .map(|dir| DirStats::collect(&dir.path).bytes)
        .collect();
    let summary = format!(
        "{} ({})",
        plural(dirs.len(), "directory", "directories"),
        human_size(sizes.iter().sum())
    );

    if !yes {
        if !io::stdin().is_terminal() {
            return Err(Error::ConfirmationRequired(summary));
        }
        let names: Vec<String> = dirs.iter().map(|dir| dir.name.to_string()).collect();
        let width = names.iter().map(|name| name.chars().count()).max();
        let width = width.unwrap_or_default();
        eprintln!("This will permanently delete:");
        for (name, size) in names.iter().zip(&sizes) {
            eprintln!("  {name:<width$}  {:>9}", human_size(*size));
        }
        if !confirm(&format!("Delete {summary}?"))? {
            eprintln!("Nothing deleted.");
            return Ok(());
        }
    }

    for dir in &dirs {
        dir.remove()?;
    }
    eprintln!("Removed {summary}.");
    Ok(())
}

fn init(shell: Option<Shell>, config: &Config) -> Result<()> {
    let shell = match shell {
        Some(shell) => shell,
        None => Shell::detect(config.shell.as_deref())?,
    };
    // Someone running `tempit init` by hand wants to know what to do with it; `eval "$(…)"`
    // wants the script itself.
    if io::stdout().is_terminal() {
        write_stdout(&shell.setup_instructions())
    } else {
        write_stdout(shell.init_script())
    }
}

fn refs(store: &Store) -> Result<()> {
    let mut out = io::stdout().lock();
    for dir in store.list()? {
        let label = dir.name.label.as_ref().map_or("", |label| label.as_str());
        writeln!(out, "{}\t{label}", dir.name.id).map_err(Error::Output)?;
    }
    Ok(())
}

/// Asks a yes/no question on the terminal. Anything but yes means no.
fn confirm(question: &str) -> Result<bool> {
    eprint!("{question} [y/N] ");
    let mut answer = String::new();
    if io::stdin()
        .lock()
        .read_line(&mut answer)
        .map_err(Error::Input)?
        == 0
    {
        // End of input (Ctrl-D): finish the prompt's line.
        eprintln!();
    }
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn print_path(path: &Path) -> Result<()> {
    write_stdout(&format!("{}\n", path.display()))
}

fn write_stdout(text: &str) -> Result<()> {
    io::stdout()
        .lock()
        .write_all(text.as_bytes())
        .map_err(Error::Output)
}

/// Prints `err` the way clap prints usage errors: `error:` with the chain of causes, then
/// what to do about it, if there is something.
fn report(err: &Error) {
    let causes = iter::successors(err.source(), |&cause| cause.source());
    let message = iter::once(err as &(dyn std::error::Error + 'static))
        .chain(causes)
        .map(describe)
        .collect::<Vec<_>>()
        .join(": ");
    let red = Style::new().bold().fg_color(Some(AnsiColor::Red.into()));
    let green = Style::new().fg_color(Some(AnsiColor::Green.into()));
    let mut stderr = anstream::stderr().lock();
    // If stderr itself is broken, there is nowhere left to report anything.
    let _ = writeln!(stderr, "{red}error:{red:#} {message}");
    if let Some(tip) = err.tip() {
        let _ = writeln!(stderr, "  {green}tip:{green:#} {tip}");
    }
}

/// Describes an error for people: OS errors lose their "(os error N)" suffix.
fn describe(err: &(dyn std::error::Error + 'static)) -> String {
    let text = err.to_string();
    err.downcast_ref::<io::Error>()
        .and_then(io::Error::raw_os_error)
        .and_then(|code| text.strip_suffix(&format!(" (os error {code})")))
        .map_or_else(|| text.clone(), str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_errors_are_described_without_their_code() {
        let err = io::Error::from_raw_os_error(2);
        assert_eq!(describe(&err), "No such file or directory");
        let custom = io::Error::other("something odd");
        assert_eq!(describe(&custom), "something odd");
    }
}
