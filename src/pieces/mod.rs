use crate::cli;
use crate::cli::AddArgs;
use crate::piece::Piece;
use crate::pieces::apt::Apt;
use crate::pieces::command::Command;
use crate::pieces::file::File;
use crate::pieces::manual::Manual;
use color_eyre::Result;
use log::{info, warn};
use serde::{Deserialize, Serialize};

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
    /// Execute the piece
    pub fn execute(&self) -> Result<()> {
        match self {
            PieceEnum::Apt(p) => p.execute(),
            PieceEnum::Command(c) => c.execute(),
            PieceEnum::File(f) => f.execute(),
            PieceEnum::Manual(m) => m.execute(),
        }
    }

    /// Undo the piece. Returns None when the undo is user-defined and has not been defined.
    pub fn undo(&self) -> Option<Result<()>> {
        match self {
            PieceEnum::Apt(p) => p.undo(),
            PieceEnum::Command(c) => c.undo(),
            PieceEnum::File(f) => f.undo(),
            PieceEnum::Manual(m) => m.undo(),
        }
    }

    /// Execute multiple pieces
    pub fn execute_bulk(pieces: Vec<&Self>) -> Result<()> {
        let (apt, command, file, manual) = Self::sort_pieces(pieces);
        Apt::execute_bulk(&apt)?;
        Command::execute_bulk(&command)?;
        File::execute_bulk(&file)?;
        Manual::execute_bulk(&manual)?;
        Ok(())
    }

    /// Undo multiple pieces.
    pub fn undo_bulk(pieces: Vec<&Self>) -> Result<()> {
        let (apt, command, file, manual) = Self::sort_pieces(pieces);
        Apt::undo_bulk(&apt)?;
        Command::undo_bulk(&command)?;
        File::undo_bulk(&file)?;
        Manual::undo_bulk(&manual)?;
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

    pub fn from_cli(args: AddArgs) -> Self {
        match args.piece {
            None => Self::from_cli_autodetect(args),
            Some(piece) => Self::from_cli_known(piece, args),
        }
    }

    fn from_cli_known(piece: cli::Piece, args: AddArgs) -> Self {
        todo!()
        // match piece {
        //     cli::Piece::Apt => PieceEnum::Apt(Apt::from_cli(args)),
        //     cli::Piece::Command => PieceEnum::Command(Command::from_cli(args)),
        //     cli::Piece::File => PieceEnum::File(File::from_cli(args)),
        //     cli::Piece::Manual => PieceEnum::Manual(Manual::from_cli(args)),
        // }
    }

    fn from_cli_autodetect(args: AddArgs) -> Self {
        let command = args.value.clone();
        match command
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .as_slice()
        {
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
