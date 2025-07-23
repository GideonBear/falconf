use crate::cli::add::add;
use crate::cli::init::init;
use crate::cli::list::list;
use crate::cli::remove::remove;
use crate::cli::sync::sync;
use crate::cli::undo::undo;
use clap::ArgAction::SetTrue;
use clap::{Args, Parser, Subcommand, ValueEnum};
use color_eyre::Result;
use expanduser::expanduser;
use log::{LevelFilter, debug};
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

mod add;
mod init;
mod list;
mod remove;
mod sync;
mod undo;

fn parse_path(s: &str) -> Result<PathBuf> {
    Ok(expanduser(s)?)
}

#[derive(Parser, Debug)]
#[command(name = "falconf", author, long_version = crate::VERSION)]
#[command(about = "TODO description")] // TODO: Edit the description here, in GitHub, in Cargo.toml
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

    /// Don't execute any commands. WARNING: this is not safe to run, as this will
    /// still make falconf think the commands were executed
    #[arg(long)]
    pub dry_run: bool,
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
    pub fn new_testing(falconf_path: PathBuf, dry_run: bool) -> Self {
        Self {
            log_level: "".to_string(),
            verbose: false,
            path: falconf_path,
            dry_run,
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
}

#[derive(Args, Debug)]
struct InitArgs {
    /// Create a new repo instead of cloning an existing one
    #[arg(long, short)]
    new: bool,

    /// The remote url
    remote: String,
}

#[derive(Args, Debug)]
struct SyncArgs {}

#[derive(ValueEnum, Copy, Clone, Debug)]
#[value(rename_all = "kebab-case")]
pub enum Piece {
    /// Executes a command in a shell. Expects a command as value.
    Command,
    /// Installs an apt package. Expects a package name as value.
    Apt,
    /// Links a file to the repo. Expects an absolute path as value.
    File,
    /// Request the user to perform an action manually *sad robot face*. Expects a message for the user (description of the action) as value.
    Manual,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// An optional comment to describe the piece for easier identification.
    #[arg(long, short)]
    pub comment: Option<String>,

    /// Omitting this argument will be interpreted as a `command` piece, but it will be translated
    /// to another piece whenever possible. For example, `falconf add apt install cowsay`
    /// will result in the same piece as `falconf add --apt cowsay`.
    #[arg(long = "piece", num_args = 1, require_equals=true, default_value_ifs=[
        ("_command", "true", "command"),
        ("_apt", "true", "apt"),
        ("_file", "true", "file"),
        ("_manual", "true", "manual"),
    ])]
    pub piece: Option<Piece>,

    /// Shorthand for `--piece=command`
    #[arg(long="command", short="c", action=SetTrue)]
    _command: (),

    /// Shorthand for `--piece=apt`
    #[arg(long="apt", action=SetTrue)]
    _apt: (),

    /// Shorthand for `--piece=file`
    #[arg(long="file", short="f", action=SetTrue)]
    _file: (),

    /// Shorthand for `--piece=manual`
    #[arg(long="manual", short="m", action=SetTrue)]
    _manual: (),

    /// The value of the piece. For example the command, the package, etc.
    /// Quoting this is optional; both `falconf add apt install cowsay` and
    /// `falconf add "apt install cowsay"` are allowed.
    #[arg(trailing_var_arg = true, required = true)]
    pub value: Vec<String>,

    /// Run the piece here (on this machine) immediately
    #[arg(long, short)]
    pub not_done_here: bool,
}

#[derive(Args, Debug)]
struct ListArgs {}

#[derive(Args, Debug)]
pub struct UndoArgs {
    #[clap(
        value_parser = parse_piece_id
    )]
    piece_id: u32,

    /// Do not undo the piece here (on this machine) immediately
    #[arg(long, short)]
    pub done_here: bool,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    #[clap(
        value_parser = parse_piece_id
    )]
    piece_ids: Vec<u32>,

    /// Remove the piece even if it is not unused
    #[arg(long, short)]
    pub force: bool,
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
    }
}
