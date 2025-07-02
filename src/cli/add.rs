use crate::cli::{AddArgs, TopLevelArgs};
use crate::execution_data::ExecutionData;
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use color_eyre::Result;

pub fn add(top_level_args: TopLevelArgs, args: AddArgs) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let execution_data = ExecutionData::new(&installation, &top_level_args)?;
    let repo = installation.repo_mut();

    // Pull the repo
    repo.pull_and_read()?;

    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    // Add the piece
    let (id, piece) = FullPiece::add(&args, &execution_data)?;
    pieces.insert(id, piece);

    // Push changes
    repo.write_and_push()?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::cli;
    use std::path::Path;

    pub fn _add_util(
        falconf_path: &Path,
        piece: cli::Piece,
        value: Vec<String>,
        comment: Option<String>,
    ) -> Result<()> {
        let top_level_args = TopLevelArgs::new_testing(falconf_path.to_path_buf(), true);

        let args = AddArgs {
            comment,
            piece: Some(piece),
            _command: (),
            _apt: (),
            _file: (),
            _manual: (),
            value,
            not_done_here: false,
        };

        add(top_level_args, args)?;

        Ok(())
    }

    pub fn add_util(falconf_path: &Path, piece: cli::Piece, value: Vec<String>) -> Result<()> {
        _add_util(falconf_path, piece, value, None)
    }

    pub fn add_util_comment(
        falconf_path: &Path,
        piece: cli::Piece,
        value: Vec<String>,
        comment: String,
    ) -> Result<()> {
        _add_util(falconf_path, piece, value, Some(comment))
    }

    // Add is tested in sync
}
