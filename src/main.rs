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

use clap::Parser;

use crate::cli::{Cli, Command};
use crate::config::Config;
use crate::error::{Error, IoContext, Result};
use crate::name::DirRef;
use crate::stats::DirStats;
use crate::store::{Store, TrackedDir};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli.command, &Config::from_env()) {
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
    match command {
        Command::Create { label } => print_path(&store()?.create(label)?.path),
        Command::List => list(&store()?),
        Command::Path { reference } => print_path(&store()?.resolve(reference.as_ref())?.path),
        Command::Remove { references } => remove(&store()?, &references),
        Command::Save {
            reference,
            destination,
        } => save(&store()?, &reference, destination, config),
        Command::Clean { yes } => clean(&store()?, yes),
        Command::Init { shell } => write_stdout(shell.init_script()),
        Command::Refs => refs(&store()?),
    }
}

fn list(store: &Store) -> Result<()> {
    let dirs = store.list()?;
    if dirs.is_empty() {
        eprintln!("No temporary directories. Create one with `tempit create [LABEL]`.");
        return Ok(());
    }
    let rows: Vec<_> = dirs
        .into_iter()
        .map(|dir| {
            let stats = DirStats::collect(&dir.path);
            (dir, stats)
        })
        .collect();
    let mut out = anstream::stdout().lock();
    render::write_table(&mut out, &rows, SystemTime::now()).map_err(Error::Output)
}

fn remove(store: &Store, references: &[DirRef]) -> Result<()> {
    // Resolve everything first, so that a typo does not leave the job half done.
    let mut dirs: Vec<TrackedDir> = Vec::new();
    for reference in references {
        let dir = store.resolve(Some(reference))?;
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    for dir in dirs {
        dir.remove()?;
        eprintln!("Removed {}", dir.path.display());
    }
    Ok(())
}

fn save(
    store: &Store,
    reference: &DirRef,
    destination: Option<PathBuf>,
    config: &Config,
) -> Result<()> {
    let dir = store.resolve(Some(reference))?;
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
    if !yes {
        let question = format!("Permanently delete {}?", count(&dirs));
        if !confirm(&question)? {
            return Err(Error::Aborted);
        }
    }
    for dir in &dirs {
        dir.remove()?;
    }
    eprintln!("Removed {}.", count(&dirs));
    Ok(())
}

fn refs(store: &Store) -> Result<()> {
    let mut out = io::stdout().lock();
    for dir in store.list()? {
        let label = dir.name.label.as_ref().map_or("", |label| label.as_str());
        writeln!(out, "{}\t{label}", dir.name.id).map_err(Error::Output)?;
    }
    Ok(())
}

/// Asks a yes/no question on the terminal. Without a terminal, refuses rather than guessing.
fn confirm(question: &str) -> Result<bool> {
    let stdin = io::stdin();
    if !stdin.is_terminal() {
        return Err(Error::ConfirmationRequired);
    }
    eprint!("{question} [y/N] ");
    let mut answer = String::new();
    stdin.lock().read_line(&mut answer).map_err(Error::Input)?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes" | "Yes" | "YES"))
}

fn count(dirs: &[TrackedDir]) -> String {
    match dirs.len() {
        1 => "1 directory".to_owned(),
        n => format!("{n} directories"),
    }
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

/// Prints `err` and its chain of causes on one line.
fn report(err: &Error) {
    let causes = iter::successors(err.source(), |&cause| cause.source());
    let message = iter::once(err as &dyn std::error::Error)
        .chain(causes)
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(": ");
    eprintln!("tempit: {message}");
}
