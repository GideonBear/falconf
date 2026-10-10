use crate::cli::PieceRef;
use crate::cli::TopLevelArgs;
use crate::execution_data::ExecutionData;
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use crate::pieces::{NonBulkPieceEnum, PieceEnum};
use clap::ArgAction::SetTrue;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt as _, eyre};
use indexmap::IndexMap;
use log::warn;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// An optional comment to describe the piece for easier identification.
    #[arg(long, conflicts_with = "remove_comment")]
    pub comment: Option<String>,

    /// Remove any existing comment
    #[arg(long, action=SetTrue, conflicts_with = "comment")]
    pub remove_comment: bool,

    /// Specify the piece id. '-' is a shortcut for the last piece.
    pub(crate) piece: PieceRef,

    // `value` is intentionally missing
    /// (command) Command to execute when undoing this
    #[arg(short, long, conflicts_with = "remove_undo")]
    pub undo: Option<String>,

    /// Remove any existing undo
    #[arg(long, action=SetTrue, conflicts_with = "undo")]
    pub remove_undo: bool,

    /// Reorder this piece, moving it after the given piece
    #[arg(long)]
    pub move_after: Option<PieceRef>,

    /// Reorder this piece, moving it before the given piece
    #[arg(long, conflicts_with = "move_after")]
    pub move_before: Option<PieceRef>,
}

#[allow(clippy::needless_pass_by_value)]
pub fn edit(top_level_args: TopLevelArgs, mut args: Args) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let execution_data = ExecutionData::new(&installation, &top_level_args)?;
    installation.pull_and_read(true, &execution_data)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    let id = args.piece.resolve(pieces)?;
    let piece = pieces.get(&id).ok_or_eyre("Piece not found")?;

    type Operation = dyn FnOnce(&mut IndexMap<u32, FullPiece>);
    let mut operations: Vec<Box<Operation>> = vec![];

    if let Some(comment) = args.comment.take() {
        operations.push(Box::new(move |pieces| {
            #[expect(clippy::missing_panics_doc, reason = "Checked above")]
            let piece = pieces.get_mut(&id).unwrap();
            if let Some(existing_comment) = &piece.comment {
                warn!("Overwriting existing comment: {existing_comment}");
            }
            piece.comment = Some(comment);
        }));
    }
    if args.remove_comment {
        if piece.comment.is_none() {
            return Err(eyre!("No comment to remove"));
        }
        operations.push(Box::new(move |pieces| {
            #[expect(clippy::missing_panics_doc, reason = "Checked above")]
            let piece = pieces.get_mut(&id).unwrap();
            piece.comment = None;
        }));
    }
    if let Some(undo) = args.undo.take() {
        if let PieceEnum::NonBulk(NonBulkPieceEnum::Command(_)) = &piece.piece {
            operations.push(Box::new(move |pieces| {
                #[expect(clippy::missing_panics_doc, reason = "Checked above")]
                let piece = pieces.get_mut(&id).unwrap();
                if let PieceEnum::NonBulk(NonBulkPieceEnum::Command(piece)) = &mut piece.piece {
                    if let Some(existing_undo) = &piece.undo_command {
                        warn!("Overwriting existing undo: {existing_undo}");
                    }
                    piece.undo_command = Some(undo);
                } else {
                    unreachable!();
                }
            }));
        } else {
            return Err(eyre!(
                "`--undo` only makes sense with a command piece. Autodetected pieces supply their own undo."
            ));
        }
    }
    if args.remove_undo {
        if let PieceEnum::NonBulk(NonBulkPieceEnum::Command(piece)) = &piece.piece {
            if piece.undo_command.is_none() {
                return Err(eyre!("No undo to remove"));
            }
            operations.push(Box::new(move |pieces| {
                #[expect(clippy::missing_panics_doc, reason = "Checked above")]
                let piece = pieces.get_mut(&id).unwrap();
                if let PieceEnum::NonBulk(NonBulkPieceEnum::Command(piece)) = &mut piece.piece {
                    piece.undo_command = None;
                } else {
                    unreachable!();
                }
            }));
        } else {
            return Err(eyre!(
                "`--remove-undo` only makes sense with a command piece. Autodetected pieces supply their own undo."
            ));
        }
    }

    let mut move_ = None;
    if let Some(move_after) = args.move_after.take() {
        move_ = Some((move_after, 0));
    }
    if let Some(move_before) = args.move_before.take() {
        move_ = Some((move_before, 1));
    }
    if let Some((move_to, modifier)) = move_ {
        let move_to = move_to.resolve(pieces)?;
        let from = pieces.get_index_of(&id).ok_or_eyre("Piece not found")?;
        let mut to = pieces
            .get_index_of(&move_to)
            .ok_or_eyre("Piece not found")?;
        if from > to {
            to += 1;
        }
        to -= modifier;
        operations.push(Box::new(move |pieces| {
            pieces.move_index(from, to);
        }))
    }

    for operation in operations {
        operation(pieces);
    }

    // Push changes
    repo.write_and_push(vec![], None)?;

    Ok(())
}

// Edit is tested manually, and partially in `list`
