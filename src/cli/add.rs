use crate::cli::{AddArgs, TopLevelArgs};
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use crate::pieces::PieceEnum;
use color_eyre::Result;

pub fn add(top_level_args: TopLevelArgs, args: AddArgs) -> Result<()> {
    // TODO
    let mut installation = Installation::get(&top_level_args)?;
    let repo = installation.repo();
    let data = repo.data();
    let pieces = data.pieces();

    // Add the piece
    pieces.push(FullPiece::from_cli(&args));

    // Push changes
    repo.write_and_push()?;

    Ok(())
}
