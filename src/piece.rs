use crate::execution_data::ExecutionData;
use color_eyre::Result;
use std::fmt::Display;

/// A single piece of configuration (non-bulk)
pub trait NonBulkPiece: Sized + Display {
    /// Execute a single piece.
    fn execute(&mut self, execution_data: &ExecutionData) -> Result<()>;

    /// Undo a single piece.
    fn undo(&mut self, execution_data: &ExecutionData) -> Result<()>;
}

/// A single piece of configuration (bulk)
pub trait BulkPiece: Sized + Display {
    /// Execute multiple of these pieces in bulk.
    fn execute_bulk(pieces: &[&mut Self], execution_data: &ExecutionData) -> Result<()>;

    /// Undo multiple of these pieces in bulk.
    fn undo_bulk(pieces: &[&mut Self], execution_data: &ExecutionData) -> Result<()>;
}
