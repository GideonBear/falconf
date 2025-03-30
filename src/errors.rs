use std::io;
use std::process::ExitStatus;

// TODO: move this to piece.rs
#[derive(Debug)]
pub enum ExecutionError {
    ProcessError(ExitStatus),
    IoError(io::Error),
    UndefinedUndo,
}

impl From<io::Error> for ExecutionError {
    fn from(err: io::Error) -> ExecutionError {
        ExecutionError::IoError(err)
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
            Err(ExecutionError::ProcessError(self))
        }
    }
}

pub trait ResultExitStatusExt {
    fn to_execution_result(self) -> ExecutionResult;
}

impl ResultExitStatusExt for io::Result<ExitStatus> {
    fn to_execution_result(self) -> ExecutionResult {
        self.map_err(ExecutionError::from)?.to_execution_result()
    }
}
