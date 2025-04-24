use crate::cli::add::add;
use crate::cli::init::init;
use crate::cli::sync::sync;
use clap::{Args, Parser, Subcommand};
use color_eyre::Result;

mod add;
mod init;
mod sync;

#[derive(Parser)]
#[command(name = "falconf", author, long_version = crate::VERSION)]
#[command(about = "TODO description")] // TODO
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Box<Commands>,
    #[clap(flatten)]
    top_level: TopLevelArgs,
}

#[derive(Args)]
struct TopLevelArgs {}

#[derive(Subcommand)]
enum Commands {
    #[command(after_help = "Intialize a new or existing Falconf repo on this machine")]
    Init(InitArgs),

    #[command(after_help = "Synchronize this machine with the repo")]
    Sync(SyncArgs),

    #[command(after_help = "Add a new piece to your configuration")]
    Add(AddArgs),
}

#[derive(Args)]
struct InitArgs {
    /// Create a new repo instead of cloning an existing one
    #[arg(long, short)]
    new: bool,

    /// The remote url
    remote: String,
}

#[derive(Args)]
struct SyncArgs {}

#[derive(Args)]
struct AddArgs {}

pub fn main() -> Result<()> {
    let cli = Cli::parse();

    match *cli.command {
        Commands::Init(args) => init(args),
        Commands::Sync(args) => sync(args),
        Commands::Add(args) => add(args),
    }
}
