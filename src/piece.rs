use crate::errors::ExecutionResult;

/// A single piece of configuration
pub trait Piece: Sized {
    /// Execute the piece
    fn execute(&self) -> ExecutionResult;

    /// Execute multiple of these pieces in bulk. Returns None when this Piece does not support it.
    fn execute_bulk(_pieces: &[&Self]) -> Option<ExecutionResult> {
        None
    }

    /// Undo the piece. Returns None when the undo is user-defined and has not been defined.
    fn undo(&self) -> Option<ExecutionResult>;

    /// Undo multiple of these pieces in bulk. Returns None when this Piece does not support it.
    fn undo_bulk(_pieces: &[&Self]) -> Option<Option<ExecutionResult>> {
        None
    }
}
