use crate::execution_data::ExecutionData;
use color_eyre::Result;
use std::fmt::Display;

/// A single piece of configuration (non-bulk)
pub trait NonBulkPiece: Sized + Display {
    /// Execute a single piece.
    fn execute(&self, execution_data: &ExecutionData) -> Result<()>;

    /// Undo a single piece. Returns None when the undo is user-defined and has not been defined.
    fn undo(&self, execution_data: &ExecutionData) -> Option<Result<()>>;
}

/// A single piece of configuration (bulk)
pub trait BulkPiece: Sized + Display {
    /// Execute multiple of these pieces in bulk.
    fn execute_bulk(pieces: &[&Self], execution_data: &ExecutionData) -> Result<()>;

    /// Undo multiple of these pieces in bulk. A BulkPiece is not allowed to have a user-defined undo.
    fn undo_bulk(pieces: &[&Self], execution_data: &ExecutionData) -> Result<()>;
}
