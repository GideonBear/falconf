use crate::cli;
use crate::cli::AddArgs;
use crate::execution_data::ExecutionData;
use crate::piece::{BulkPiece, NonBulkPiece};
use crate::pieces::apt::Apt;
use crate::pieces::command::Command;
use crate::pieces::file::File;
use crate::pieces::manual::Manual;
use color_eyre::Result;
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
    fn execute(&self, execution_data: &ExecutionData) -> Result<()> {
        match self {
            NonBulkPieceEnum::Command(command) => command.execute(execution_data),
            NonBulkPieceEnum::File(file) => file.execute(execution_data),
            NonBulkPieceEnum::Manual(manual) => manual.execute(execution_data),
        }
    }

    fn undo(&self, execution_data: &ExecutionData) -> Option<Result<()>> {
        match self {
            NonBulkPieceEnum::Command(command) => command.undo(execution_data),
            NonBulkPieceEnum::File(file) => file.undo(execution_data),
            NonBulkPieceEnum::Manual(manual) => manual.undo(execution_data),
        }
    }
}

impl PieceEnum {
    // TODO: maybe deduplicate between execute and undo?
    // TODO: Improve naming
    /// Execute multiple pieces
    pub fn execute_bulk<F: FnMut()>(
        pieces: Vec<(&Self, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        if execution_data.dry_run {
            warn!("Dry run! Not doing anything.");
            return Ok(());
        }
        let (apt, non_bulk) = Self::sort_pieces(pieces);
        Self::execute_bulk_bulk(apt, execution_data)?;
        Self::execute_non_bulk_bulk(non_bulk, execution_data)?;
        Ok(())
    }

    fn execute_bulk_bulk<F: FnMut(), P: BulkPiece>(
        pieces: Vec<(&P, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        if !pieces.is_empty() {
            let (pieces, cbs): (Vec<&P>, Vec<F>) = pieces.into_iter().unzip();
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
        pieces: Vec<(&NonBulkPieceEnum, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        for (piece, mut cb) in pieces {
            if !execution_data.test_run {
                piece.execute(execution_data)?;
            } else {
                warn!("Test run! Refraining from execution, but marking as normal.");
            }
            cb();
        }
        Ok(())
    }

    // TODO: deduplicate
    /// Undo multiple pieces.
    pub fn undo_bulk<F: FnMut()>(
        pieces: Vec<(&Self, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        if execution_data.dry_run {
            warn!("Dry run! Not doing anything.");
            return Ok(());
        }
        let (apt, non_bulk) = Self::sort_pieces(pieces);
        Self::undo_bulk_bulk(apt, execution_data)?;
        Self::undo_non_bulk_bulk(non_bulk, execution_data)?;
        Ok(())
    }

    fn undo_bulk_bulk<F: FnMut(), P: BulkPiece>(
        pieces: Vec<(&P, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        if !pieces.is_empty() {
            let (pieces, cbs): (Vec<&P>, Vec<F>) = pieces.into_iter().unzip();
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
        pieces: Vec<(&NonBulkPieceEnum, F)>,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        for (piece, mut cb) in pieces {
            if !execution_data.test_run {
                match piece.undo(execution_data) {
                    None => {
                        // TODO: Flag to add undo parameter
                        todo!("Undefined undo for piece; we should prompt then retry with that");
                    }
                    Some(Err(e)) => return Err(e),
                    Some(Ok(())) => {}
                }
            } else {
                warn!("Test run! Refraining from execution, but marking as normal.");
            }
            cb();
        }
        Ok(())
    }

    #[allow(clippy::type_complexity)] // This is pretty clean
    pub fn sort_pieces<F: FnMut()>(
        pieces: Vec<(&Self, F)>,
    ) -> (Vec<(&Apt, F)>, Vec<(&NonBulkPieceEnum, F)>) {
        #[expect(unused_parens)]
        let (mut apt) = (vec![]);
        let mut non_bulk = vec![];
        for piece in pieces {
            match piece {
                (PieceEnum::Bulk(BulkPieceEnum::Apt(p)), cb) => apt.push((p, cb)),
                (PieceEnum::NonBulk(piece), cb) => non_bulk.push((piece, cb)),
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
            cli::Piece::Apt => PieceEnum::Bulk(BulkPieceEnum::Apt(Apt::from_cli(args)?)),
            cli::Piece::Command => {
                PieceEnum::NonBulk(NonBulkPieceEnum::Command(Command::from_cli(args)?))
            }
            cli::Piece::File => PieceEnum::NonBulk(NonBulkPieceEnum::File(File::from_cli(args)?)),
            cli::Piece::Manual => {
                PieceEnum::NonBulk(NonBulkPieceEnum::Manual(Manual::from_cli(args)))
            }
        })
    }

    fn from_cli_autodetect(args: &AddArgs) -> Result<Self> {
        let command = args.value.clone();
        Ok(
            match command
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<&str>>()
                .as_slice()
            {
                // TODO: test
                ["apt", "install", package]
                | ["apt", "install", package, "-y"]
                | ["apt", "install", "-y", package]
                | ["apt", "-y", "install", package] => {
                    info!("Using `apt` piece instead of `command`");
                    PieceEnum::Bulk(BulkPieceEnum::Apt(Apt::from_cli_autodetected(
                        args,
                        package.to_string(),
                    )))
                }
                ["apt", ..] => unknown!("apt", "apt", args),
                ["ln", ..] => unknown!("ln", "file", args),
                _ => PieceEnum::NonBulk(NonBulkPieceEnum::Command(Command::from_cli(args)?)),
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
