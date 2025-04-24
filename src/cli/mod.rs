use crate::cli::add::add;
use crate::cli::init::init;
use crate::cli::sync::sync;
use clap::ArgAction::SetTrue;
use clap::{Args, Parser, Subcommand, ValueEnum};
use color_eyre::Result;
use log::{LevelFilter, debug};
use std::str::FromStr;

mod add;
mod init;
mod sync;

#[derive(Parser, Debug)]
#[command(name = "falconf", author, long_version = crate::VERSION)]
#[command(about = "TODO description")] // TODO
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Box<Commands>,
    #[clap(flatten)]
    top_level: TopLevelArgs,
}

#[derive(Args, Debug)]
struct TopLevelArgs {
    /// The log level to use.
    #[arg(long, short, default_value = "info")]
    log_level: String,

    /// Output debug logs. Alias for `--log-level debug`.
    #[arg(long, short)]
    verbose: bool,
}

impl TopLevelArgs {
    fn effective_log_level(&self) -> &str {
        if self.verbose {
            "debug"
        } else {
            &self.log_level
        }
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(after_help = "Intialize a new or existing Falconf repo on this machine")]
    Init(InitArgs),

    #[command(after_help = "Synchronize this machine with the repo")]
    Sync(SyncArgs),

    #[command(after_help = "Add a new piece to your configuration")]
    Add(AddArgs),
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
enum Piece {
    /// Executes a command in a shell. Expects a command as value.
    Command,
    /// Installs an apt package. Expects a package name as value.
    Apt,
    /// Tracks a file. Expects an absolute path as value.
    File,
    /// Tracks a manual action. Expects a message for the user (description of the action) as value.
    Manual,
}

#[derive(Args, Debug)]
struct AddArgs {
    /// Omitting this argument will be interpreted as a `command` piece, but it will be translated
    /// to another piece whenever possible. For example, `falconf add apt install cowsay`
    /// will result in the same piece as `falconf add --apt cowsay`.
    #[arg(long = "piece", num_args = 1, require_equals=true, default_value_ifs=[
        ("_command", "true", "command"),
        ("_apt", "true", "apt"),
        ("_file", "true", "file"),
        ("_manual", "true", "manual"),
    ])]
    pub(crate) piece: Option<Piece>,

    /// Shorthand for `--piece=command`
    #[arg(long="command", action=SetTrue)]
    _command: (),

    /// Shorthand for `--piece=apt`
    #[arg(long="apt", action=SetTrue)]
    _apt: (),

    /// Shorthand for `--piece=file`
    #[arg(long="file", action=SetTrue)]
    _file: (),

    /// Shorthand for `--piece=manual`
    #[arg(long="manual", action=SetTrue)]
    _manual: (),

    /// The value of the piece. For example the command, the package, etc.
    /// Quoting this is optional; both `falconf add apt install cowsay` and
    /// `falconf add "apt install cowsay"` are allowed.
    #[arg(trailing_var_arg = true)]
    value: Vec<String>,
}

pub fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(LevelFilter::from_str(cli.top_level.effective_log_level())?)
        .init();

    debug!("{cli:?}");

    match *cli.command {
        Commands::Init(args) => init(args),
        Commands::Sync(args) => sync(args),
        Commands::Add(args) => add(args),
    }
}
