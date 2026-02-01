use crate::cli;
use crate::cli::AddArgs;
use crate::execution_data::ExecutionData;
use crate::piece::{BulkPiece, NonBulkPiece};
use crate::pieces::apt::Apt;
use crate::pieces::command::Command;
use crate::pieces::file::File;
use crate::pieces::manual::Manual;
use crate::utils::print_id;
use color_eyre::Result;
use itertools::Itertools;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub mod apt;
pub mod command;
pub mod file;
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
        PieceEnum::NonBulk(NonBulkPieceEnum::Command(Command::from_cli($args)?))
    }};
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PieceEnum {
    Bulk(BulkPieceEnum),
    NonBulk(NonBulkPieceEnum),
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BulkPieceEnum {
    Apt(Apt),
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NonBulkPieceEnum {
    Command(Command),
    File(File),
    Manual(Manual),
}

impl NonBulkPieceEnum {
    fn execute(&mut self, execution_data: &ExecutionData) -> Result<()> {
        match self {
            Self::Command(command) => command.execute(execution_data),
            Self::File(file) => file.execute(execution_data),
            Self::Manual(manual) => manual.execute(execution_data),
        }
    }

    fn undo(&mut self, execution_data: &ExecutionData) -> Result<()> {
        match self {
            Self::Command(command) => command.undo(execution_data),
            Self::File(file) => file.undo(execution_data),
            Self::Manual(manual) => manual.undo(execution_data),
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
        let (apt, non_bulk) = Self::sort_pieces(pieces);
        Self::execute_bulk_bulk(apt, execution_data)?;
        Self::execute_non_bulk_bulk(non_bulk, execution_data)?;
        Ok(())
    }

    fn execute_bulk_bulk<F: FnMut(), P: BulkPiece>(
        pieces: Vec<(u32, &mut P, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        if !pieces.is_empty() {
            info!("Executing multiple pieces at once:");
            for (id, piece, _cb) in &pieces {
                info!("- {} {piece}", print_id(*id));
            }
            let (_ids, pieces, cbs): (Vec<u32>, Vec<&mut P>, Vec<F>) =
                pieces.into_iter().multiunzip();
            // As we're executing in bulk, we want to wait with the callbacks until after execution
            if !execution_data.test_run {
                P::execute_bulk(&pieces, execution_data)?;
            } else {
                warn!("Test run! Refraining from execution, but marking as normal.");
            }
            for mut cb in cbs {
                cb();
            }
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
        let (apt, non_bulk) = Self::sort_pieces(pieces);
        Self::undo_bulk_bulk(apt, execution_data)?;
        Self::undo_non_bulk_bulk(non_bulk, execution_data)?;
        Ok(())
    }

    fn undo_bulk_bulk<F: FnMut(), P: BulkPiece>(
        pieces: Vec<(u32, &mut P, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        if !pieces.is_empty() {
            info!("Undoing multiple pieces at once:");
            for (id, piece, _cb) in &pieces {
                info!("- {} {piece}", print_id(*id));
            }
            let (_ids, pieces, cbs): (Vec<u32>, Vec<&mut P>, Vec<F>) =
                pieces.into_iter().multiunzip();
            // As we're executing in bulk, we want to wait with the callbacks until after execution
            if !execution_data.test_run {
                P::undo_bulk(&pieces, execution_data)?;
            } else {
                warn!("Test run! Refraining from execution, but marking as normal.");
            }
            for mut cb in cbs {
                cb();
            }
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
        Vec<(u32, &mut NonBulkPieceEnum, F)>,
    ) {
        #[expect(unused_parens)]
        let (mut apt) = (vec![]);
        let mut non_bulk = vec![];
        for (id, piece, cb) in pieces {
            match piece {
                Self::Bulk(BulkPieceEnum::Apt(p)) => apt.push((id, p, cb)),
                Self::NonBulk(piece) => non_bulk.push((id, piece, cb)),
            }
        }
        (apt, non_bulk)
    }

    pub fn from_cli(args: &AddArgs) -> Result<Self> {
        Ok(match args.piece {
            None => Self::from_cli_autodetect(args)?,
            Some(piece) => Self::from_cli_known(piece, args)?,
        })
    }

    fn from_cli_known(piece: cli::Piece, args: &AddArgs) -> Result<Self> {
        Ok(match piece {
            cli::Piece::Apt => Self::Bulk(BulkPieceEnum::Apt(Apt::from_cli(args)?)),
            cli::Piece::Command => {
                Self::NonBulk(NonBulkPieceEnum::Command(Command::from_cli(args)?))
            }
            cli::Piece::File => Self::NonBulk(NonBulkPieceEnum::File(File::from_cli(args)?)),
            cli::Piece::Manual => Self::NonBulk(NonBulkPieceEnum::Manual(Manual::from_cli(args))),
        })
    }

    fn from_cli_autodetect(args: &AddArgs) -> Result<Self> {
        let command = args.value.clone();
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
                _ => Self::NonBulk(NonBulkPieceEnum::Command(Command::from_cli(args)?)),
            },
        )
    }
}

impl Display for BulkPieceEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apt(piece) => piece.fmt(f),
        }
    }
}

impl Display for NonBulkPieceEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(piece) => piece.fmt(f),
            Self::File(piece) => piece.fmt(f),
            Self::Manual(piece) => piece.fmt(f),
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
