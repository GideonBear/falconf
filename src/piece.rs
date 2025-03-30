use crate::pieces::file::FileError;
use std::io;
use std::process::ExitStatus;

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

#[derive(Debug)]
pub enum ExecutionError {
    Process(ExitStatus),
    IoError(io::Error),
    UndefinedUndo,
    File(FileError),
}

impl From<io::Error> for ExecutionError {
    fn from(err: io::Error) -> ExecutionError {
        ExecutionError::IoError(err)
    }
}

impl From<FileError> for ExecutionError {
    fn from(err: FileError) -> ExecutionError {
        ExecutionError::File(err)
    }
}

pub type ExecutionResult = Result<(), ExecutionError>;

pub trait ExitStatusExt {
    fn to_execution_result(self) -> ExecutionResult;
}

impl ExitStatusExt for ExitStatus {
    fn to_execution_result(self) -> ExecutionResult {
        if self.success() {
            Ok(())
        } else {
            Err(ExecutionError::Process(self))
        }
    }
}

pub trait ResultExitStatusExt {
    fn to_execution_result(self) -> ExecutionResult;
}

impl ResultExitStatusExt for io::Result<ExitStatus> {
    fn to_execution_result(self) -> ExecutionResult {
        self?.to_execution_result()
    }
}
