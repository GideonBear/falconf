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
use std::fmt::Display;

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
        PieceEnum::Command(Command::from_cli($args)?)
    }};
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PieceEnum {
    Apt(Apt),
    Command(Command),
    File(File),
    Manual(Manual),
}

impl PieceEnum {
    // TODO: looks to be unnecessary?
    // /// Execute a single piece. Should not be called.
    // pub fn _execute(&self) -> Result<()> {
    //     match self {
    //         PieceEnum::Apt(p) => p._execute(),
    //         PieceEnum::Command(c) => c._execute(),
    //         PieceEnum::File(f) => f._execute(),
    //         PieceEnum::Manual(m) => m._execute(),
    //     }
    // }
    //
    // /// Undo a single piece. Should not be called. Returns None when the undo is user-defined and has not been defined.
    // pub fn _undo(&self) -> Option<Result<()>> {
    //     match self {
    //         PieceEnum::Apt(piece) => piece._undo(),
    //         PieceEnum::Command(piece) => piece._undo(),
    //         PieceEnum::File(piece) => piece._undo(),
    //         PieceEnum::Manual(piece) => piece._undo(),
    //     }
    // }

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
        let (apt, command, file, manual) = Self::sort_pieces(pieces);
        Self::execute_bulk_bulk(apt, execution_data)?;
        Self::execute_non_bulk_bulk(command, execution_data)?;
        Self::execute_non_bulk_bulk(file, execution_data)?;
        Self::execute_non_bulk_bulk(manual, execution_data)?;
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

    fn execute_non_bulk_bulk<F: FnMut(), P: NonBulkPiece>(
        pieces: Vec<(&P, F)>,
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
        let (apt, command, file, manual) = Self::sort_pieces(pieces);
        Self::undo_bulk_bulk(apt, execution_data)?;
        Self::undo_non_bulk_bulk(command, execution_data)?;
        Self::undo_non_bulk_bulk(file, execution_data)?;
        Self::undo_non_bulk_bulk(manual, execution_data)?;
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

    fn undo_non_bulk_bulk<F: FnMut(), P: NonBulkPiece>(
        pieces: Vec<(&P, F)>,
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
    ) -> (
        Vec<(&Apt, F)>,
        Vec<(&Command, F)>,
        Vec<(&File, F)>,
        Vec<(&Manual, F)>,
    ) {
        let (mut apt, mut command, mut file, mut manual) = (vec![], vec![], vec![], vec![]);
        for piece in pieces {
            match piece {
                (PieceEnum::Apt(p), cb) => apt.push((p, cb)),
                (PieceEnum::Command(c), cb) => command.push((c, cb)),
                (PieceEnum::File(f), cb) => file.push((f, cb)),
                (PieceEnum::Manual(m), cb) => manual.push((m, cb)),
            }
        }
        (apt, command, file, manual)
    }

    pub fn from_cli(args: &AddArgs) -> Result<Self> {
        Ok(match args.piece {
            None => Self::from_cli_autodetect(args)?,
            Some(piece) => Self::from_cli_known(piece, args)?,
        })
    }

    fn from_cli_known(piece: cli::Piece, args: &AddArgs) -> Result<Self> {
        Ok(match piece {
            cli::Piece::Apt => PieceEnum::Apt(Apt::from_cli(args)?),
            cli::Piece::Command => PieceEnum::Command(Command::from_cli(args)?),
            cli::Piece::File => PieceEnum::File(File::from_cli(args)?),
            cli::Piece::Manual => PieceEnum::Manual(Manual::from_cli(args)),
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
                    PieceEnum::Apt(Apt::from_cli_autodetected(args, package.to_string()))
                }
                ["apt", ..] => unknown!("apt", "apt", args),
                ["ln", ..] => unknown!("ln", "file", args),
                _ => PieceEnum::Command(Command::from_cli(args)?),
            },
        )
    }
}

impl Display for PieceEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceEnum::Apt(piece) => piece.fmt(f),
            PieceEnum::Command(piece) => piece.fmt(f),
            PieceEnum::File(piece) => piece.fmt(f),
            PieceEnum::Manual(piece) => piece.fmt(f),
        }
    }
}
