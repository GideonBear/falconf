use crate::cli;
use crate::cli::add;
use crate::execution_data::ExecutionData;
use crate::piece::{BulkPiece, NonBulkPiece as _, Piece};
use crate::pieces::apt::Apt;
use crate::pieces::cargo_install::CargoInstall;
use crate::pieces::command::Command;
use crate::pieces::deb_get::DebGet;
use crate::pieces::falconf_init::FalconfInit;
use crate::pieces::file::File;
use crate::pieces::gsettings::Gsettings;
use crate::pieces::manual::Manual;
use crate::utils::print_id;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use itertools::Itertools as _;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::Path;

pub mod apt;
pub mod cargo_install;
pub mod command;
pub mod deb_get;
pub mod falconf_init;
pub mod file;
pub mod gsettings;
pub mod manual;

macro_rules! unknown {
    ($command:expr, $target:expr, $args:expr) => {{
        warn!(concat!(
            "Unknown `",
            $command,
            "` command, using 'command' (instead of '",
            $target,
            "')"
        ));
        PieceEnum::NonBulk(NonBulkPieceEnum::Command(Command::from_cli($args)))
    }};
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub enum PieceEnum {
    Bulk(BulkPieceEnum),
    NonBulk(NonBulkPieceEnum),
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub enum BulkPieceEnum {
    Apt(Apt),
    DebGet(DebGet),
    CargoInstall(CargoInstall),
}

impl BulkPieceEnum {
    fn seed(&self, execution_data: &ExecutionData) -> Result<(String, bool, Option<String>)> {
        match self {
            Self::Apt(apt) => apt.seed(execution_data),
            Self::DebGet(deb_get) => deb_get.seed(execution_data),
            Self::CargoInstall(cargo_install) => cargo_install.seed(execution_data),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub enum NonBulkPieceEnum {
    Command(Command),
    File(File),
    Manual(Manual),
    Gsettings(Gsettings),
    FalconfInit(FalconfInit),
}

impl NonBulkPieceEnum {
    fn execute(&mut self, execution_data: &ExecutionData) -> Result<()> {
        match self {
            Self::Command(command) => command.execute(execution_data),
            Self::File(file) => file.execute(execution_data),
            Self::Manual(manual) => manual.execute(execution_data),
            Self::Gsettings(gsettings) => gsettings.execute(execution_data),
            Self::FalconfInit(falconf_init) => falconf_init.execute(execution_data),
        }
    }

    fn undo(&mut self, execution_data: &ExecutionData) -> Result<()> {
        match self {
            Self::Command(command) => command.undo(execution_data),
            Self::File(file) => file.undo(execution_data),
            Self::Manual(manual) => manual.undo(execution_data),
            Self::Gsettings(gsettings) => gsettings.undo(execution_data),
            Self::FalconfInit(falconf_init) => falconf_init.undo(execution_data),
        }
    }

    fn seed(&self, execution_data: &ExecutionData) -> Result<(String, bool, Option<String>)> {
        match self {
            Self::Command(command) => command.seed(execution_data),
            Self::File(file) => file.seed(execution_data),
            Self::Manual(manual) => manual.seed(execution_data),
            Self::Gsettings(gsettings) => gsettings.seed(execution_data),
            Self::FalconfInit(falconf_init) => falconf_init.seed(execution_data),
        }
    }
}

impl PieceEnum {
    // TODO(low): maybe deduplicate between execute and undo with some generics or something?
    // TODO(low): Improve naming
    /// Execute multiple pieces
    pub fn execute_bulk<F: FnMut()>(
        pieces: Vec<(u32, &mut Self, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        // if execution_data.dry_run {
        //     warn!("Dry run! Not doing anything.");
        //     return Ok(());
        // }
        let (apt, deb_get, cargo_install, non_bulk) = Self::sort_pieces(pieces);
        Self::execute_bulk_bulk(apt, execution_data)?;
        Self::execute_bulk_bulk(deb_get, execution_data)?;
        Self::execute_bulk_bulk(cargo_install, execution_data)?;
        Self::execute_non_bulk_bulk(non_bulk, execution_data)?;
        Ok(())
    }

    fn execute_bulk_bulk<F: FnMut(), P: BulkPiece>(
        pieces: Vec<(u32, &mut P, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        if pieces.is_empty() {
            return Ok(());
        }

        if pieces.len() == 1 {
            let (id, piece, _cb) = &pieces[0];
            info!("Executing piece: {} {piece}", print_id(*id));
        } else {
            info!("Executing multiple pieces at once:");
            for (id, piece, _cb) in &pieces {
                info!("- {} {piece}", print_id(*id));
            }
        }

        let (_ids, pieces, cbs): (Vec<u32>, Vec<&mut P>, Vec<F>) = pieces.into_iter().multiunzip();
        // As we're executing in bulk, we want to wait with the callbacks until after execution
        if !execution_data.test_run {
            P::execute_bulk(&pieces, execution_data)?;
        } else {
            warn!("Test run! Refraining from execution, but marking as normal.");
        }
        for mut cb in cbs {
            cb();
        }

        Ok(())
    }

    fn execute_non_bulk_bulk<F: FnMut()>(
        pieces: Vec<(u32, &mut NonBulkPieceEnum, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        for (id, piece, mut cb) in pieces {
            info!("Executing piece: {} {piece}", print_id(id));
            if !execution_data.test_run {
                piece.execute(execution_data)?;
            } else {
                warn!("Test run! Refraining from execution, but marking as normal.");
            }
            cb();
        }
        Ok(())
    }

    /// Undo multiple pieces.
    pub fn undo_bulk<F: FnMut()>(
        pieces: Vec<(u32, &mut Self, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        // if execution_data.dry_run {
        //     warn!("Dry run! Not doing anything.");
        //     return Ok(());
        // }
        let (apt, deb_get, cargo_install, non_bulk) = Self::sort_pieces(pieces);
        Self::undo_bulk_bulk(apt, execution_data)?;
        Self::undo_bulk_bulk(deb_get, execution_data)?;
        Self::undo_bulk_bulk(cargo_install, execution_data)?;
        Self::undo_non_bulk_bulk(non_bulk, execution_data)?;
        Ok(())
    }

    fn undo_bulk_bulk<F: FnMut(), P: BulkPiece>(
        pieces: Vec<(u32, &mut P, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        if pieces.is_empty() {
            return Ok(());
        }

        if pieces.len() == 1 {
            let (id, piece, _cb) = &pieces[0];
            info!("Undoing piece: {} {piece}", print_id(*id));
        } else {
            info!("Undoing multiple pieces at once:");
            for (id, piece, _cb) in &pieces {
                info!("- {} {piece}", print_id(*id));
            }
        }

        let (_ids, pieces, cbs): (Vec<u32>, Vec<&mut P>, Vec<F>) = pieces.into_iter().multiunzip();
        // As we're executing in bulk, we want to wait with the callbacks until after execution
        if !execution_data.test_run {
            P::undo_bulk(&pieces, execution_data)?;
        } else {
            warn!("Test run! Refraining from execution, but marking as normal.");
        }
        for mut cb in cbs {
            cb();
        }

        Ok(())
    }

    fn undo_non_bulk_bulk<F: FnMut()>(
        pieces: Vec<(u32, &mut NonBulkPieceEnum, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        for (id, piece, mut cb) in pieces {
            info!("Undoing piece: {} {piece}", print_id(id));
            if !execution_data.test_run {
                piece.undo(execution_data)?;
            } else {
                warn!("Test run! Refraining from execution, but marking as normal.");
            }
            cb();
        }
        Ok(())
    }

    #[expect(clippy::type_complexity)] // This is pretty clean
    pub fn sort_pieces<F: FnMut()>(
        pieces: Vec<(u32, &mut Self, F)>,
    ) -> (
        Vec<(u32, &mut Apt, F)>,
        Vec<(u32, &mut DebGet, F)>,
        Vec<(u32, &mut CargoInstall, F)>,
        Vec<(u32, &mut NonBulkPieceEnum, F)>,
    ) {
        let mut apt = vec![];
        let mut deb_get = vec![];
        let mut cargo_install = vec![];
        let mut non_bulk = vec![];
        for (id, piece, cb) in pieces {
            match piece {
                Self::Bulk(BulkPieceEnum::Apt(p)) => apt.push((id, p, cb)),
                Self::Bulk(BulkPieceEnum::DebGet(p)) => deb_get.push((id, p, cb)),
                Self::Bulk(BulkPieceEnum::CargoInstall(p)) => cargo_install.push((id, p, cb)),
                Self::NonBulk(piece) => non_bulk.push((id, piece, cb)),
            }
        }
        (apt, deb_get, cargo_install, non_bulk)
    }

    pub fn from_cli(args: &add::Args) -> Result<Self> {
        Ok(match args.piece {
            None => Self::from_cli_autodetect(args)?,
            Some(piece) => Self::from_cli_known(piece, args)?,
        })
    }

    fn from_cli_known(piece: cli::Piece, args: &add::Args) -> Result<Self> {
        Ok(match piece {
            cli::Piece::Apt => Self::Bulk(BulkPieceEnum::Apt(Apt::from_cli(args)?)),
            cli::Piece::Command => {
                Self::NonBulk(NonBulkPieceEnum::Command(Command::from_cli(args)))
            }
            cli::Piece::File => Self::NonBulk(NonBulkPieceEnum::File(File::from_cli(args)?)),
            cli::Piece::Manual => Self::NonBulk(NonBulkPieceEnum::Manual(Manual::from_cli(args))),
            cli::Piece::Gsettings => {
                Self::NonBulk(NonBulkPieceEnum::Gsettings(Gsettings::from_cli(args)?))
            }
            cli::Piece::DebGet => Self::Bulk(BulkPieceEnum::DebGet(DebGet::from_cli(args)?)),
            cli::Piece::CargoInstall => {
                Self::Bulk(BulkPieceEnum::CargoInstall(CargoInstall::from_cli(args)?))
            }
        })
    }

    fn from_cli_autodetect(args: &add::Args) -> Result<Self> {
        let mut command = args.value.clone();
        // When the piece is known, we leave the value alone
        //  When the piece is unknown, we assume it's a command.
        //  If there's only a single value, split it so we can do proper autodetection.
        if command.len() == 1 {
            command = shell_words::split(&command[0])?;
        }
        Ok(
            match command
                .iter()
                .map(String::as_str)
                .collect::<Vec<&str>>()
                .as_slice()
            {
                // TODO(test): test
                ["apt", "install", package]
                | ["apt", "install", package, "-y"]
                | ["apt", "install", "-y", package]
                | ["apt", "-y", "install", package] => {
                    info!("Using `apt` piece instead of `command`");
                    Self::Bulk(BulkPieceEnum::Apt(Apt::from_cli_autodetected(
                        args,
                        package.to_string(),
                    )))
                }
                ["apt", ..] => unknown!("apt", "apt", args),
                ["ln", ..] => unknown!("ln", "file", args),
                [path] if Path::new(path).exists() => {
                    return Err(eyre!(
                        "Refusing to add command piece which is also a valid path. \
                        Please use -f if you meant to add a file, or -c to explicitly add a command."
                    ));
                }
                ["gsettings", "set", schema, key, value] => {
                    info!("Using `gsettings` piece instead of `command`");
                    Self::NonBulk(NonBulkPieceEnum::Gsettings(Gsettings::new(
                        schema.to_string(),
                        key.to_string(),
                        value.to_string(),
                    )?))
                }
                ["gsettings", ..] => unknown!("gsettings", "gsettings", args),
                ["deb-get", "install", package] => {
                    info!("Using `deb-get` piece instead of `command`");
                    Self::Bulk(BulkPieceEnum::DebGet(DebGet::from_cli_autodetected(
                        args,
                        package.to_string(),
                    )))
                }
                ["deb-get", ..] => unknown!("deb-get", "deb-get", args),
                ["cargo", "install" | "binstall", crate_]
                | ["cargo", "install" | "binstall", "-y", crate_]
                | ["cargo", "install" | "binstall", crate_, "-y"] => {
                    info!("Using `cargo-install` piece instead of `command`");
                    Self::Bulk(BulkPieceEnum::CargoInstall(
                        CargoInstall::from_cli_autodetected(args, crate_.to_string()),
                    ))
                }
                ["cargo", "install", ..] => unknown!("cargo install", "cargo-install", args),
                ["cargo", "binstall", ..] => unknown!("cargo binstall", "cargo-install", args),
                _ => Self::NonBulk(NonBulkPieceEnum::Command(Command::from_cli(args))),
            },
        )
    }

    pub fn seed(&self, execution_data: &ExecutionData) -> Result<(String, bool, Option<String>)> {
        match self {
            Self::Bulk(piece) => piece.seed(execution_data),
            Self::NonBulk(piece) => piece.seed(execution_data),
        }
    }
}

impl Display for BulkPieceEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apt(piece) => piece.fmt(f),
            Self::DebGet(piece) => piece.fmt(f),
            Self::CargoInstall(piece) => piece.fmt(f),
        }
    }
}

impl Display for NonBulkPieceEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(piece) => piece.fmt(f),
            Self::File(piece) => piece.fmt(f),
            Self::Manual(piece) => piece.fmt(f),
            Self::Gsettings(piece) => piece.fmt(f),
            Self::FalconfInit(piece) => piece.fmt(f),
        }
    }
}

impl Display for PieceEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bulk(piece) => piece.fmt(f),
            Self::NonBulk(piece) => piece.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::add::tests::add_args_util;

    #[test]
    fn test_from_cli_autodetect_works_split() -> Result<()> {
        let args = add_args_util(
            None,
            vec![
                "apt".to_string(),
                "install".to_string(),
                "rolldice".to_string(),
            ],
            None,
        );
        let piece = PieceEnum::from_cli(&args)?;
        assert!(matches!(piece, PieceEnum::Bulk(BulkPieceEnum::Apt(_))));

        Ok(())
    }

    #[test]
    fn test_from_cli_autodetect_works_combined() -> Result<()> {
        let args = add_args_util(None, vec!["apt install rolldice".to_string()], None);
        let piece = PieceEnum::from_cli(&args)?;
        assert!(matches!(piece, PieceEnum::Bulk(BulkPieceEnum::Apt(_))));

        Ok(())
    }
}
