use crate::cli::TopLevelArgs;
use crate::cli::parse_piece_id;
use crate::execution_data::ExecutionData;
use crate::installation::Installation;
use clap::Args;
use color_eyre::Result;
use color_eyre::eyre::OptionExt;

// TODO: support hier ook meerdere piece ids
#[derive(Args, Debug)]
pub struct UndoArgs {
    #[clap(
        value_parser = parse_piece_id
    )]
    piece_id: u32,

    /// Do not undo the piece here (on this machine) immediately
    #[arg(long, short)]
    pub done_here: bool,
}

pub fn undo(top_level_args: TopLevelArgs, args: UndoArgs) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let execution_data = ExecutionData::new(&installation, &top_level_args)?;
    installation.pull_and_read(true)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    let piece = pieces
        .get_mut(&args.piece_id)
        .ok_or_eyre("Piece id not found")?;

    piece.undo(args.piece_id, &args, &execution_data)?;

    // Push changes
    repo.write_and_push(vec![])?;

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
            done_here: true,
        };

        undo(top_level_args, args)?;

        Ok(())
    }

    // Undo is tested in sync
}
