use crate::cli::{TopLevelArgs, UndoArgs};
use crate::execution_data::ExecutionData;
use crate::installation::Installation;
use color_eyre::Result;
use color_eyre::eyre::OptionExt;

pub fn undo(top_level_args: TopLevelArgs, args: UndoArgs) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let execution_data = ExecutionData::new(&installation, &top_level_args)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    let piece = pieces
        .get_mut(&args.piece_id)
        .ok_or_eyre("Piece id not found")?;

    piece.undo(&args, &execution_data)?;

    // Push changes
    repo.write_and_push()?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use std::path::Path;

    pub fn undo_util(falconf_path: &Path, id: u32) -> Result<()> {
        let top_level_args = TopLevelArgs::new_testing(falconf_path.to_path_buf(), true);

        let args = UndoArgs {
            piece_id: id,
            not_done_here: false,
        };

        undo(top_level_args, args)?;

        Ok(())
    }

    pub fn test_undo() -> Result<()> {
        todo!()
    }
}
