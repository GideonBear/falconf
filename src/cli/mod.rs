use crate::cli::add::add;
use crate::cli::init::init;
use crate::cli::list::list;
use crate::cli::remove::remove;
use crate::cli::sync::sync;
use crate::cli::undo::undo;
use clap::{Args, Parser, Subcommand};
use color_eyre::Result;
use expanduser::expanduser;
use init::InitArgs;
use list::ListArgs;
use log::{LevelFilter, debug};
use remove::RemoveArgs;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use sync::SyncArgs;

mod add;
pub use add::AddArgs;
pub use add::Piece;
pub(crate) mod init;
mod list;
mod push;
mod remove;
mod sync;
mod undo;

use crate::cli::push::{PushArgs, push};
pub use undo::UndoArgs;

fn parse_path(s: &str) -> Result<PathBuf> {
    Ok(expanduser(s)?)
}

#[derive(Parser, Debug)]
#[command(name = "falconf", version)]
#[command(about = "TODO description")] // TODO(med): Edit the description here, in GitHub, in Cargo.toml
pub struct Cli {
    #[command(subcommand)]
    command: Box<Commands>,
    #[clap(flatten)]
    pub top_level: TopLevelArgs,
}

#[derive(Args, Debug, Clone)]
pub struct TopLevelArgs {
    /// The log level to use.
    #[arg(long, short, default_value = "info")]
    log_level: String,

    /// Output debug logs. Alias for `--log-level debug`.
    #[arg(long, short)]
    verbose: bool,

    /// The path to the falconf directory.
    #[arg(long, short, default_value = "~/.falconf", value_parser = parse_path)]
    pub path: PathBuf,

    // /// Don't execute any commands, and do not mark pieces as executed. WARNING: this
    // /// is not a complete dry run, and certain commands (like push) will still retain
    // /// their functionality. It does guarantee the data file isn't changed.
    // #[arg(long, short)]
    // pub dry_run: bool,
    /// Don't execute any commands, but mark pieces as executed. WARNING: this
    /// is not safe to use, and is meant for testing purposes only.
    #[arg(long)]
    pub test_run: bool,
}

impl TopLevelArgs {
    fn effective_log_level(&self) -> &str {
        if self.verbose {
            "debug"
        } else {
            &self.log_level
        }
    }

    #[cfg(test)]
    pub fn new_testing(falconf_path: PathBuf, test_run: bool) -> Self {
        Self {
            log_level: "".to_string(),
            verbose: false,
            path: falconf_path,
            // dry_run: false,
            test_run,
        }
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Intialize a new or existing Falconf repo on this machine")]
    Init(InitArgs),

    #[command(about = "Synchronize changes in the repo to this machine")]
    Sync(SyncArgs),

    #[command(about = "Add a new piece to your configuration")]
    Add(AddArgs),

    #[command(about = "List all pieces")]
    List(ListArgs),

    #[command(about = "Undo a piece")]
    Undo(UndoArgs),

    #[command(about = "Remove a piece")]
    Remove(RemoveArgs),

    #[command(about = "Push changes in files to the repo")]
    Push(PushArgs),
}

fn parse_piece_id(s: &str) -> Result<u32, String> {
    if s.len() != 8 {
        return Err("Value must be exactly 8 hex digits".to_string());
    }
    u32::from_str_radix(s, 16).map_err(|_| "Invalid hex format".to_string())
}

pub fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(LevelFilter::from_str(cli.top_level.effective_log_level())?)
        .init();

    debug!("{cli:?}");

    let Cli { command, top_level } = cli;

    match *command {
        Commands::Init(args) => init(top_level, args),
        Commands::Sync(args) => sync(top_level, args),
        Commands::Add(args) => add(top_level, args),
        Commands::List(args) => list(top_level, args, &mut io::stdout().lock()),
        Commands::Undo(args) => undo(top_level, args),
        Commands::Remove(args) => remove(top_level, args),
        Commands::Push(args) => push(top_level, args),
    }
}
