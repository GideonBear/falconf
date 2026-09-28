use crate::cli::parse_piece_ref;
use crate::cli::{PieceRef, TopLevelArgs};
use crate::execution_data::ExecutionData;
use crate::installation::Installation;
use color_eyre::Result;
use color_eyre::eyre::OptionExt;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Specify the piece id. '-' is a shortcut for the last piece.
    #[clap(value_parser = parse_piece_ref)]
    pub(crate) piece: PieceRef,
}

#[allow(clippy::needless_pass_by_value)]
pub fn done(top_level_args: TopLevelArgs, args: Args) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let execution_data = ExecutionData::new(&installation, &top_level_args)?;
    installation.pull_and_read(true)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();
    let id = args.piece.resolve(pieces)?;
    let piece = pieces.get_mut(&id).ok_or_eyre("Piece not found")?;

    piece.done(execution_data.machine);

    // Push changes
    repo.write_and_push(vec![])?;

    Ok(())
}
