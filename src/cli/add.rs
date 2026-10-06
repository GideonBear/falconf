use crate::cli::TopLevelArgs;
use crate::execution_data::ExecutionData;
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use crate::pieces::{NonBulkPieceEnum, PieceEnum};
use clap::ArgAction::SetTrue;
use clap::ValueEnum;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use log::info;
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
    /// Changes a gsettings/dconf value. Expects a schema, key, and value as value.
    Gsettings,
    /// Installs a deb-get package. Expects a package name as value.
    DebGet,
    /// Installs a cargo crate using `cargo install`, or `cargo binstall` if available. Expects a crate name as value.
    CargoInstall,
}

#[derive(clap::Args, Debug)]
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
        ("_gsettings", "true", "gsettings"),
        ("_deb_get", "true", "deb-get"),
    ])]
    pub piece: Option<Piece>,

    /// Alias for `--piece=command`
    #[arg(long="command", short='c', action=SetTrue)]
    _command: (),

    /// Alias for `--piece=apt`
    #[arg(long="apt", action=SetTrue)]
    _apt: (),

    /// Alias for `--piece=file`
    #[arg(long="file", short='f', action=SetTrue)]
    _file: (),

    /// Alias for `--piece=manual`
    #[arg(long="manual", short='m', action=SetTrue)]
    _manual: (),

    /// Alias for `--piece=gsettings`
    #[arg(long="gsettings", short='g', action=SetTrue)]
    _gsettings: (),

    /// Alias for `--piece=deb-get`
    #[arg(long="deb-get", action=SetTrue)]
    _deb_get: (),

    /// Alias for `--piece=cargo-install`
    #[arg(long="cargo-install", action=SetTrue)]
    _cargo_install: (),

    /// The value of the piece. For example the command, the package, etc.
    /// Quoting this is optional; both `falconf add apt install cowsay` and
    /// `falconf add "apt install cowsay"` are allowed.
    #[arg(trailing_var_arg = true, required = true)]
    pub value: Vec<String>,

    /// (command) Command to execute when undoing this
    #[arg(short, long)]
    pub undo: Option<String>,

    /// Assume this piece is already executed on this machine
    #[arg(long, short)]
    pub done: bool,
}

#[allow(clippy::needless_pass_by_value)]
pub fn add(top_level_args: TopLevelArgs, args: Args) -> Result<()> {
    let piece = FullPiece::from_cli(&args)?;

    let is_file = piece.file().is_some();
    if args.done && is_file {
        return Err(eyre!(
            "The concept of '--done' is incompatible with file pieces. Adding a file piece performs a special action."
        ));
    }

    if args.undo.is_some()
        && !matches!(
            piece.piece,
            PieceEnum::NonBulk(NonBulkPieceEnum::Command(_))
        )
    {
        return Err(eyre!(
            "`--undo` only makes sense with a command piece. Autodetected pieces supply their own undo."
        ));
    }

    add_internal(&top_level_args, vec![(piece, args.done)])
}

pub fn add_internal(top_level_args: &TopLevelArgs, to_add: Vec<(FullPiece, bool)>) -> Result<()> {
    let mut installation = Installation::get(top_level_args)?;
    let execution_data = ExecutionData::new(&installation, top_level_args)?;
    installation.pull_and_read(true)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    let mut files = vec![];
    for (mut piece, done) in to_add {
        // Add the piece
        let id = piece.add(&execution_data, done)?;
        if let Some(file) = piece.file().map(Path::to_path_buf) {
            files.push(file);
        }
        info!("Added: {}", piece.print(id));
        pieces.insert(id, piece);
    }

    // Push changes
    repo.write_and_push(files, None)?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::path::Path;

    pub fn add_args_util(
        piece: Option<Piece>,
        value: Vec<String>,
        comment: Option<String>,
    ) -> Args {
        Args {
            comment,
            piece,
            _command: (),
            _apt: (),
            _file: (),
            _manual: (),
            _gsettings: (),
            _deb_get: (),
            _cargo_install: (),
            value,
            undo: None,
            done: false,
        }
    }

    fn add_util_opts(
        falconf_path: &Path,
        piece: Piece,
        value: Vec<String>,
        test_run: bool,
        comment: Option<String>,
    ) -> Result<()> {
        let top_level_args = TopLevelArgs::new_testing(falconf_path.to_path_buf(), test_run);

        let args = add_args_util(Some(piece), value, comment);

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
