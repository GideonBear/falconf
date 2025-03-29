use std::io;
use std::process::ExitStatus;

#[derive(Debug)]
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
        self.map_err(ExecutionError::from)?.to_execution_result()
    }
}
