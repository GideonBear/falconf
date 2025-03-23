use crate::machine::Machine;
use crate::pieces::PieceEnum;

pub(crate) struct FullPiece {
    piece: PieceEnum,
    done_on: Vec<Machine>,
    undo: bool,
    undone_on: Vec<Machine>,
}
