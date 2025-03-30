use crate::errors::ExecutionResult;

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

    /// Undo multiple of these pieces in bulk.
    fn undo_bulk(pieces: &[&Self]) -> Option<ExecutionResult> {
        for piece in pieces {
            match piece.undo() {
                None => return None,
                Some(Err(e)) => return Some(Err(e)),
                Some(Ok(())) => {}
            }
        }
        Some(Ok(()))
    }
}
