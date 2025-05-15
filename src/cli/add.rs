use crate::cli::{AddArgs, TopLevelArgs};
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use color_eyre::Result;
use log::debug;

pub fn add(top_level_args: TopLevelArgs, mut args: AddArgs) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    // Make sure that if a single value is passed (e.g. in quotes, or not via a shell)
    // we still get the expected result
    if args.value.len() == 1 {
        args.value = args.value[0]
            .split_whitespace()
            .map(|x| x.to_owned())
            .collect();

        debug!("Corrected args.value to {:?}", args.value);
    }

    // Add the piece
    pieces.push(FullPiece::from_cli(args)?);

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

    pub fn add_util_file(falconf_path: &Path, file: String) -> Result<()> {
        debug!("Adding File piece: {file}");

        let top_level_args = TopLevelArgs::new_testing(falconf_path.to_path_buf());

        let args = AddArgs {
            comment: None,
            piece: Some(cli::Piece::File),
            _command: (),
            _apt: (),
            _file: (),
            _manual: (),
            value: vec![file],
        };

        add(top_level_args, args)?;

        Ok(())
    }
}
