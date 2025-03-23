use crate::piece::Piece;

struct FullPiece<T: Piece> {
    piece: T,
    done_on: Vec<Machine>,
    undo: bool,
    undone_on: Vec<Machine>,
}
