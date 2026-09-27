use crate::cli::TopLevelArgs;
use crate::cli::{PieceRef, parse_piece_ref};
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use crate::pieces::{NonBulkPieceEnum, PieceEnum};
use clap::ArgAction::SetTrue;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt as _, eyre};
use log::{info, warn};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// An optional comment to describe the piece for easier identification.
    #[arg(long, conflicts_with = "remove_comment")]
    pub comment: Option<String>,

    /// Remove any existing comment
    #[arg(long, action=SetTrue, conflicts_with = "comment")]
    pub remove_comment: bool,

    /// Specify the piece id. '-' is a shortcut for the last piece.
    #[clap(value_parser = parse_piece_ref)]
    pub(crate) piece: PieceRef,

    // `value` is intentionally missing
    /// (command) Command to execute when undoing this
    #[arg(short, long, conflicts_with = "remove_undo")]
    pub undo: Option<String>,

    /// Remove any existing undo
    #[arg(long, action=SetTrue, conflicts_with = "undo")]
    pub remove_undo: bool,

    /// Reorder this piece, moving it after the given piece
    #[arg(long, value_parser = parse_piece_ref)]
    pub move_after: Option<PieceRef>,
}

#[allow(clippy::needless_pass_by_value)]
pub fn edit(top_level_args: TopLevelArgs, mut args: Args) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    installation.pull_and_read(true)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    let id = args.piece.resolve(pieces)?;

    // Mutate pieces before fetching the piece
    if let Some(move_after) = args.move_after.take() {
        let move_after = move_after.resolve(pieces)?;
        let from = pieces.get_index_of(&id).ok_or_eyre("Piece not found")?;
        let mut to = pieces
            .get_index_of(&move_after)
            .ok_or_eyre("Piece not found")?;
        if from > to {
            to += 1;
        }
        pieces.move_index(from, to);
    }

    let piece = pieces.get_mut(&id).ok_or_eyre("Piece not found")?;

    type Operation = dyn FnOnce(&mut FullPiece) -> Result<()>;
    let mut operations: Vec<Box<Operation>> = vec![];

    if let Some(comment) = args.comment.take() {
        operations.push(Box::new(|piece| {
            if let Some(existing_comment) = &piece.comment {
                warn!("Overwriting existing comment: {existing_comment}");
            }
            piece.comment = Some(comment);
            Ok(())
        }));
    }
    if args.remove_comment {
        operations.push(Box::new(|piece| {
            if piece.comment.is_none() {
                return Err(eyre!("No comment to remove"));
            }
            piece.comment = None;
            Ok(())
        }));
    }
    if let Some(undo) = args.undo.take() {
        operations.push(Box::new(|piece| {
            if let PieceEnum::NonBulk(NonBulkPieceEnum::Command(piece)) = &mut piece.piece {
                if let Some(existing_undo) = &piece.undo_command {
                    warn!("Overwriting existing undo: {existing_undo}");
                }
                piece.undo_command = Some(undo);
                Ok(())
            } else {
                Err(eyre!("`--undo` only makes sense with a command piece. Autodetected pieces supply their own undo."))
            }
        }));
    }
    if args.remove_undo {
        operations.push(Box::new(|piece| {
             if let PieceEnum::NonBulk(NonBulkPieceEnum::Command(piece)) = &mut piece.piece {
                if piece.undo_command.is_none() {
                    return Err(eyre!("No undo to remove"));
                }
                piece.undo_command = None;
                Ok(())
            } else {
                Err(eyre!("`--remove-undo` only makes sense with a command piece. Autodetected pieces supply their own undo."))
            }
        }));
    }

    for operation in operations {
        if let Err(err) = operation(piece) {
            info!("Found error during edit; writing and pushing the changes that *were* done");
            repo.write_and_push(vec![])?;
            return Err(err);
        }
    }

    // Push changes
    repo.write_and_push(vec![])?;

    Ok(())
}

// Edit is tested manually, and partially in `list`
