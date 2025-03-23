use std::io;
use std::process::ExitStatus;
pub(crate) enum ExecutionError {
    ProcessError(ExitStatus),
    IoError(io::Error),
}

impl From<io::Error> for ExecutionError {
    fn from(err: io::Error) -> ExecutionError {
        ExecutionError::IoError(err)
    }
}

pub(crate) type ExecutionResult = Result<(), ExecutionError>;

pub(crate) trait ExitStatusExt {
    fn to_execution_result(self) -> ExecutionResult;
}

impl ExitStatusExt for ExitStatus {
    fn to_execution_result(self) -> ExecutionResult {
        if self.success() {
            Ok(())
        } else {
            Err(ExecutionError::ProcessError(self))
        }
    }
}

pub(crate) trait ResultExitStatusExt {
    fn to_execution_result(self) -> ExecutionResult;
}

impl ResultExitStatusExt for io::Result<ExitStatus> {
    fn to_execution_result(self) -> ExecutionResult {
        self
            .map_err(ExecutionError::from)?
            .to_execution_result()
    }
}

/// A single piece of configuration
pub(crate) trait Piece: Sized {
    /// Execute the piece
    fn execute(&self)-> ExecutionResult;

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
