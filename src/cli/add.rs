use crate::cli::TopLevelArgs;
use crate::execution_data::ExecutionData;
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use clap::ArgAction::SetTrue;
use clap::{Args, ValueEnum};
use color_eyre::Result;

#[derive(ValueEnum, Copy, Clone, Debug)]
#[value(rename_all = "kebab-case")]
pub enum Piece {
    /// Executes a command in a shell. Expects a command as value.
    Command,
    /// Installs an apt package. Expects a package name as value.
    Apt,
    /// Links a file to the repo. Expects an absolute path as value.
    File,
    /// Request the user to perform an action manually *sad robot face*. Expects a message for the user (description of the action) as value.
    Manual,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// An optional comment to describe the piece for easier identification.
    #[arg(long, short)]
    pub comment: Option<String>,

    /// Omitting this argument will be interpreted as a `command` piece, but it will be translated
    /// to another piece whenever possible. For example, `falconf add apt install cowsay`
    /// will result in the same piece as `falconf add --apt cowsay`.
    #[arg(long = "piece", num_args = 1, require_equals=true, default_value_ifs=[
        ("_command", "true", "command"),
        ("_apt", "true", "apt"),
        ("_file", "true", "file"),
        ("_manual", "true", "manual"),
    ])]
    pub piece: Option<Piece>,

    /// Shorthand for `--piece=command`
    #[arg(long="command", short='c', action=SetTrue)]
    _command: (),

    /// Shorthand for `--piece=apt`
    #[arg(long="apt", action=SetTrue)]
    _apt: (),

    /// Shorthand for `--piece=file`
    #[arg(long="file", short='f', action=SetTrue)]
    _file: (),

    /// Shorthand for `--piece=manual`
    #[arg(long="manual", short='m', action=SetTrue)]
    _manual: (),

    /// The value of the piece. For example the command, the package, etc.
    /// Quoting this is optional; both `falconf add apt install cowsay` and
    /// `falconf add "apt install cowsay"` are allowed.
    #[arg(trailing_var_arg = true, required = true)]
    pub value: Vec<String>,

    /// Run the piece here (on this machine) immediately
    #[arg(long, short)]
    pub not_done_here: bool,
}

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
    let file = piece.file().map(|p| p.to_path_buf());
    pieces.insert(id, piece);

    // Push changes
    repo.write_and_push(match file {
        None => vec![],
        Some(file) => vec![file.to_path_buf()],
    })?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::cli::add;
    use std::path::Path;

    pub fn _add_util(
        falconf_path: &Path,
        piece: Piece,
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

    pub fn add_util(falconf_path: &Path, piece: Piece, value: Vec<String>) -> Result<()> {
        _add_util(falconf_path, piece, value, None)
    }

    pub fn add_util_comment(
        falconf_path: &Path,
        piece: Piece,
        value: Vec<String>,
        comment: String,
    ) -> Result<()> {
        _add_util(falconf_path, piece, value, Some(comment))
    }

    // Add is tested in sync
}
