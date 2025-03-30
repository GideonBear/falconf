use crate::errors::{ExecutionError, ExecutionResult};

/// A single piece of configuration
pub trait Piece: Sized {
    /// Execute the piece
    fn execute(&self) -> ExecutionResult;

    /// Execute multiple of these pieces in bulk.
    fn execute_bulk(pieces: &[&Self]) -> ExecutionResult {
        for piece in pieces {
            piece.execute()?
        }
        Ok(())
    }

    /// Undo the piece. Returns None when the undo is user-defined and has not been defined.
    fn undo(&self) -> Option<ExecutionResult>;

    /// Undo multiple of these pieces in bulk. We assume that a Piece that has a bulk undo can never have a user-defined undo.
    fn undo_bulk(pieces: &[&Self]) -> ExecutionResult {
        for piece in pieces {
            match piece.undo() {
                None => return Err(ExecutionError::UndefinedUndo),
                Some(Err(e)) => return Err(e),
                Some(Ok(())) => {}
            }
        }
        Ok(())
    }
}
