use crate::cli::complete;
use crate::cli::{PieceRef, TopLevelArgs};
use crate::execution_context::ExecutionContext;
use crate::full_piece::{FullPiece, Todo};
use crate::installation::Installation;
use crate::utils::print_id_raw;
use clap_complete::ArgValueCandidates;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, eyre};
use log::info;
use std::collections::HashSet;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Specify piece ids. '-' is a shortcut for the last piece.
    #[clap(required = true)]
    #[arg(add = ArgValueCandidates::new(complete::pieces))]
    pub(crate) pieces: Vec<PieceRef>,

    /// Execute already-executed pieces
    #[arg(long)]
    redo: bool,
}

#[allow(clippy::needless_pass_by_value)]
pub fn exec(top_level_args: TopLevelArgs, args: Args) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let ctx = ExecutionContext::new(&installation, &top_level_args)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let (pieces, _machines) = data.as_mut_parts();

    let piece_ids = args
        .pieces
        .iter()
        .map(|x| x.resolve(pieces))
        .collect::<Result<HashSet<_>>>()?;

    for &id in piece_ids.iter() {
        let piece = pieces.get(&id).ok_or_eyre("Piece not found")?;
        if let Todo::Noop = piece.todo(&ctx)
            && !args.redo
        {
            return Err(eyre!(
                "Piece {} is not pending execution or undo, but --redo was not passed",
                print_id_raw(id)
            ));
        }
    }

    let mut to_execute = vec![];
    let mut to_undo = vec![];

    for (&id, piece) in pieces {
        if !piece_ids.contains(&id) {
            continue;
        }
        if ctx.machine_data.in_group(ctx.machine, &piece.group) {
            to_execute.push((id, piece));
        } else {
            to_undo.push((id, piece));
        }
    }
    if let Err(err) = FullPiece::do_(to_execute, to_undo, &ctx) {
        info!("Found error during exec; writing and pushing the changes that *were* done");
        repo.write_and_push(vec![], None)?;
        return Err(err);
    }

    // Push changes
    repo.write_and_push(vec![], None)?;

    Ok(())
}
