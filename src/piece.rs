use crate::execution_context::ExecutionContext;
use color_eyre::Result;
use std::fmt::Display;

pub trait Piece: Display {
    fn seed(&self, ctx: &ExecutionContext) -> Result<(String, bool, Option<String>)>;
}

/// A single piece of configuration (non-bulk)
pub trait NonBulkPiece: Piece {
    /// Execute a single piece.
    fn execute(&mut self, ctx: &ExecutionContext) -> Result<()>;

    /// Undo a single piece.
    fn undo(&mut self, ctx: &ExecutionContext) -> Result<()>;
}

/// A single piece of configuration (bulk)
pub trait BulkPiece: Piece {
    /// Execute multiple of these pieces in bulk.
    fn execute_bulk(pieces: &[&mut Self], ctx: &ExecutionContext) -> Result<()>;

    /// Undo multiple of these pieces in bulk.
    fn undo_bulk(pieces: &[&mut Self], ctx: &ExecutionContext) -> Result<()>;
}
