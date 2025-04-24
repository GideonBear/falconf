use crate::cli::{AddArgs, Cli};
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use crate::pieces::PieceEnum;
use color_eyre::Result;

pub fn add(cli: &Cli, args: &AddArgs) -> Result<()> {
    // TODO
    // piece_enum: PieceEnum,
    // comment: Option<String>,
    let mut installation = Installation::get(cli)?;
    let repo = installation.repo();
    let mut data = repo.data();
    let mut pieces = data.pieces();

    // Add the piece
    // TODO
    // pieces.push(FullPiece::new(piece_enum, comment));

    // Push changes
    repo.write_and_push()?;

    Ok(())
}
