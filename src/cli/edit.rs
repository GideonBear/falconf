use crate::cli::TopLevelArgs;
use crate::cli::parse_piece_id;
use crate::execution_data::ExecutionData;
use crate::installation::Installation;
use clap::ArgAction::SetTrue;
use clap::Args;
use color_eyre::Result;

#[derive(Args, Debug)]
pub struct EditArgs {
    /// An optional comment to describe the piece for easier identification.
    #[arg(long)]
    pub comment: Option<String>,

    /// Remove any existing comment
    #[arg(long, action=SetTrue)]
    pub remove_comment: bool,

    #[clap(value_parser = parse_piece_id)]
    pub(crate) piece_id: u32,

    /// The value of the piece. For example the command, the package, etc.
    /// Quoting this is optional; both `falconf add apt install cowsay` and
    /// `falconf add "apt install cowsay"` are allowed.
    #[arg(trailing_var_arg = true)]
    pub value: Option<Vec<String>>,

    // TODO: this should also be checked to be only on commands
    /// (command) Command to execute when undoing this
    #[arg(short, long)]
    pub undo: Option<String>,

    /// Remove any existing undo
    #[arg(long, action=SetTrue)]
    pub remove_undo: bool,
}

pub fn edit(top_level_args: TopLevelArgs, args: EditArgs) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let execution_data = ExecutionData::new(&installation, &top_level_args)?;
    installation.pull_and_read(true)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    // Add the piece
    // let (id, piece) = FullPiece::add(&args, &execution_data)?;
    // let file = piece.file().map(|p| p.to_path_buf());
    // pieces.insert(id, piece);
    //
    // // Push changes
    // repo.write_and_push(match file {
    //     None => vec![],
    //     Some(file) => vec![file],
    // })?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // TODO(test): test
}
