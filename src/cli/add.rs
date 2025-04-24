use crate::full_piece::FullPiece;
use crate::installation::Installation;
use crate::pieces::PieceEnum;
use color_eyre::Result;

pub fn add(
    installation: &mut Installation,
    piece_enum: PieceEnum,
    comment: Option<String>,
) -> Result<()> {
    let repo = installation.repo();
    let mut data = repo.data();
    let mut pieces = data.pieces();

    // Add the piece
    pieces.push(FullPiece::new(piece_enum, comment));

    // Push changes
    repo.write_and_push()?;

    Ok(())
}
