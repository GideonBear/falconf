use crate::cli::PieceRef;
use crate::cli::TopLevelArgs;
use crate::execution_context::ExecutionContext;
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use log::info;
use std::collections::{HashMap, HashSet};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Specify piece ids. '-' is a shortcut for the last piece.
    #[clap(required = true)]
    pieces: Vec<PieceRef>,
}

#[allow(clippy::needless_pass_by_value)]
pub fn undo(top_level_args: TopLevelArgs, args: Args) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let ctx = ExecutionContext::new(&installation, &top_level_args)?;
    installation.pull_and_read(true, &ctx)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    let piece_ids = args
        .pieces
        .iter()
        .map(|x| x.resolve(pieces))
        .collect::<Result<HashSet<_>>>()?;

    let pieces_to_undo: HashMap<u32, &mut FullPiece> = pieces
        .iter_mut()
        .filter(|(k, _v)| piece_ids.contains(k))
        .map(|(k, v)| (*k, v))
        .collect();
    if pieces_to_undo.keys().copied().collect::<HashSet<_>>() != piece_ids {
        return Err(eyre!("Piece not found"));
    }

    // TODO(low): This should be bulk. If it shouldn't, there should be a comment explaining why
    for (id, piece) in pieces_to_undo {
        if let Err(err) = piece.undo(id, &ctx) {
            info!("Found error during undo; writing and pushing the changes that *were* done");
            repo.write_and_push(vec![], None)?;
            return Err(err);
        }
    }

    // Push changes
    repo.write_and_push(vec![], None)?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::path::Path;

    pub fn undo_util(falconf_path: &Path, piece: PieceRef) -> Result<()> {
        let top_level_args = TopLevelArgs::new_testing(falconf_path.to_path_buf(), true);

        let args = Args {
            pieces: vec![piece],
        };

        undo(top_level_args, args)?;

        Ok(())
    }

    // Undo is tested in sync
}
