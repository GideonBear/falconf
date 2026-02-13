use crate::cli::TopLevelArgs;
use crate::execution_data::ExecutionData;
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use clap::ArgAction::SetTrue;
use clap::ValueEnum;
use color_eyre::Result;
use std::path::Path;

#[derive(ValueEnum, Copy, Clone, Debug)]
#[value(rename_all = "kebab-case")]
pub enum Piece {
    /// Executes a command in a shell. Expects a command as value.
    Command,
    /// Installs an apt package. Expects a package name as value.
    Apt,
    /// Links a file to the repo. Expects a path (absolute or relative) as value.
    File,
    /// Request the user to perform an action manually *sad robot face*. Expects a message for the user (description of the action) as value.
    Manual,
}

#[derive(clap::Args, Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "Will set `piece`, should not be read directly"
)]
pub struct Args {
    /// An optional comment to describe the piece for easier identification.
    #[arg(long)]
    pub comment: Option<String>,

    /// Omitting this argument will be interpreted as a `command` piece, but it will be translated
    /// to another piece whenever possible. For example, `falconf add apt install cowsay`
    /// will result in the same piece as `falconf add --apt cowsay`.
    #[arg(long = "piece", num_args = 1, require_equals=true, default_value_ifs=[
        ("_command", "true", "command"),
        ("_apt", "true", "apt"),
        ("_file", "true", "file"),
        ("_manual", "true", "manual"),
    ], group="piece_group")]
    pub piece: Option<Piece>,

    /// Alias for `--piece=command`
    #[arg(long="command", short='c', action=SetTrue, group="piece_group")]
    _command: (),

    /// Alias for `--piece=apt`
    #[arg(long="apt", action=SetTrue, group="piece_group")]
    _apt: (),

    /// Alias for `--piece=file`
    #[arg(long="file", short='f', action=SetTrue, group="piece_group")]
    _file: (),

    /// Alias for `--piece=manual`
    #[arg(long="manual", short='m', action=SetTrue, group="piece_group")]
    _manual: (),

    /// The value of the piece. For example the command, the package, etc.
    /// Quoting this is optional; both `falconf add apt install cowsay` and
    /// `falconf add "apt install cowsay"` are allowed.
    #[arg(trailing_var_arg = true, required = true)]
    pub value: Vec<String>,

    /// (command) Command to execute when undoing this
    #[arg(short, long)]
    pub undo: Option<String>,

    /// Run the piece here (on this machine) immediately
    #[arg(long, short)]
    pub not_done_here: bool,
}

#[allow(clippy::needless_pass_by_value)]
pub fn add(top_level_args: TopLevelArgs, args: Args) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let execution_data = ExecutionData::new(&installation, &top_level_args)?;
    installation.pull_and_read(true)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    // Add the piece
    let (id, piece) = FullPiece::add(&args, &execution_data)?;
    let file = piece.file().map(Path::to_path_buf);
    pieces.insert(id, piece);

    // Push changes
    repo.write_and_push(file.map_or_else(Vec::new, |file| vec![file]))?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use std::path::Path;

    fn add_util_opts(
        falconf_path: &Path,
        piece: Piece,
        value: Vec<String>,
        test_run: bool,
        comment: Option<String>,
    ) -> Result<()> {
        let top_level_args = TopLevelArgs::new_testing(falconf_path.to_path_buf(), test_run);

        let args = Args {
            comment,
            piece: Some(piece),
            _command: (),
            _apt: (),
            _file: (),
            _manual: (),
            value,
            undo: None,
            not_done_here: false,
        };

        add(top_level_args, args)?;

        Ok(())
    }

    pub fn add_util(falconf_path: &Path, piece: Piece, value: Vec<String>) -> Result<()> {
        add_util_opts(falconf_path, piece, value, true, None)
    }

    pub fn add_util_no_test_run(
        falconf_path: &Path,
        piece: Piece,
        value: Vec<String>,
    ) -> Result<()> {
        add_util_opts(falconf_path, piece, value, false, None)
    }

    pub fn add_util_comment(
        falconf_path: &Path,
        piece: Piece,
        value: Vec<String>,
        comment: String,
    ) -> Result<()> {
        add_util_opts(falconf_path, piece, value, true, Some(comment))
    }

    // Add is tested in sync
}
