use crate::cli;
use crate::cli::AddArgs;
use crate::execution_data::ExecutionData;
use crate::piece::Piece;
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
        PieceEnum::Command(Command::from_cli($args))
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

    /// Execute multiple pieces
    pub fn execute_bulk(pieces: Vec<&Self>, execution_data: &ExecutionData) -> Result<()> {
        if execution_data.dry_run {
            warn!("Dry run! Not doing anything.");
            return Ok(());
        }
        let (apt, command, file, manual) = Self::sort_pieces(pieces);
        if !apt.is_empty() {
            Apt::execute_bulk(&apt, execution_data)?;
        }
        if !command.is_empty() {
            Command::execute_bulk(&command, execution_data)?;
        }
        if !file.is_empty() {
            File::execute_bulk(&file, execution_data)?;
        }
        if !manual.is_empty() {
            Manual::execute_bulk(&manual, execution_data)?;
        }
        Ok(())
    }

    /// Undo multiple pieces.
    pub fn undo_bulk(pieces: Vec<&Self>, execution_data: &ExecutionData) -> Result<()> {
        if execution_data.dry_run {
            warn!("Dry run! Not doing anything.");
            return Ok(());
        }
        let (apt, command, file, manual) = Self::sort_pieces(pieces);
        if !apt.is_empty() {
            Apt::undo_bulk(&apt, execution_data)?;
        }
        if !command.is_empty() {
            Command::undo_bulk(&command, execution_data)?;
        }
        if !file.is_empty() {
            File::undo_bulk(&file, execution_data)?;
        }
        if !manual.is_empty() {
            Manual::undo_bulk(&manual, execution_data)?;
        }
        Ok(())
    }

    pub fn sort_pieces(pieces: Vec<&Self>) -> (Vec<&Apt>, Vec<&Command>, Vec<&File>, Vec<&Manual>) {
        let (mut apt, mut command, mut file, mut manual) = (vec![], vec![], vec![], vec![]);
        for piece in pieces {
            match piece {
                PieceEnum::Apt(p) => apt.push(p),
                PieceEnum::Command(c) => command.push(c),
                PieceEnum::File(f) => file.push(f),
                PieceEnum::Manual(m) => manual.push(m),
            }
        }
        (apt, command, file, manual)
    }

    pub fn from_cli(args: &AddArgs) -> Result<Self> {
        Ok(match args.piece {
            None => Self::from_cli_autodetect(args),
            Some(piece) => Self::from_cli_known(piece, args)?,
        })
    }

    fn from_cli_known(piece: cli::Piece, args: &AddArgs) -> Result<Self> {
        Ok(match piece {
            cli::Piece::Apt => PieceEnum::Apt(Apt::from_cli(args)?),
            cli::Piece::Command => PieceEnum::Command(Command::from_cli(args)),
            cli::Piece::File => PieceEnum::File(File::from_cli(args)?),
            cli::Piece::Manual => PieceEnum::Manual(Manual::from_cli(args)),
        })
    }

    fn from_cli_autodetect(args: &AddArgs) -> Self {
        let command = args.value.clone();
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
            _ => PieceEnum::Command(Command::from_cli(args)),
        }
    }
}

impl Display for PieceEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PieceEnum::Apt(piece) => piece.to_string(),
                PieceEnum::Command(piece) => piece.to_string(),
                PieceEnum::File(piece) => piece.to_string(),
                PieceEnum::Manual(piece) => piece.to_string(),
            }
        )
    }
}
