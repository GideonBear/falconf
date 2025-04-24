use crate::piece::Piece;
use crate::pieces::apt_package::AptPackage;
use crate::pieces::command::Command;
use crate::pieces::file::File;
use crate::pieces::manual::Manual;
use color_eyre::Result;
use serde::{Deserialize, Serialize};

pub mod apt_package;
pub mod command;
pub mod file;
pub mod manual;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PieceEnum {
    AptPackage(AptPackage),
    Command(Command),
    File(File),
    Manual(Manual),
}

impl PieceEnum {
    /// Execute the piece
    pub fn execute(&self) -> Result<()> {
        match self {
            PieceEnum::AptPackage(p) => p.execute(),
            PieceEnum::Command(c) => c.execute(),
            PieceEnum::File(f) => f.execute(),
            PieceEnum::Manual(m) => m.execute(),
        }
    }

    /// Undo the piece. Returns None when the undo is user-defined and has not been defined.
    pub fn undo(&self) -> Option<Result<()>> {
        match self {
            PieceEnum::AptPackage(p) => p.undo(),
            PieceEnum::Command(c) => c.undo(),
            PieceEnum::File(f) => f.undo(),
            PieceEnum::Manual(m) => m.undo(),
        }
    }

    /// Execute multiple pieces
    pub fn execute_bulk(pieces: Vec<&Self>) -> Result<()> {
        let (apt_package, command, file, manual) = Self::sort_pieces(pieces);
        AptPackage::execute_bulk(&apt_package)?;
        Command::execute_bulk(&command)?;
        File::execute_bulk(&file)?;
        Manual::execute_bulk(&manual)?;
        Ok(())
    }

    /// Undo multiple pieces.
    pub fn undo_bulk(pieces: Vec<&Self>) -> Result<()> {
        let (apt_package, command, file, manual) = Self::sort_pieces(pieces);
        AptPackage::undo_bulk(&apt_package)?;
        Command::undo_bulk(&command)?;
        File::undo_bulk(&file)?;
        Manual::undo_bulk(&manual)?;
        Ok(())
    }

    pub fn sort_pieces(
        pieces: Vec<&Self>,
    ) -> (Vec<&AptPackage>, Vec<&Command>, Vec<&File>, Vec<&Manual>) {
        let (mut apt_package, mut command, mut file, mut manual) = (vec![], vec![], vec![], vec![]);
        for piece in pieces {
            match piece {
                PieceEnum::AptPackage(p) => apt_package.push(p),
                PieceEnum::Command(c) => command.push(c),
                PieceEnum::File(f) => file.push(f),
                PieceEnum::Manual(m) => manual.push(m),
            }
        }
        (apt_package, command, file, manual)
    }
}
