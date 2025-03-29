use crate::errors::{ExecutionResult, ResultExitStatusExt};
use crate::logging::CommandExt;
use crate::piece::Piece;
use std::process;

/// Install a package with apt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AptPackage {
    /// The package to install
    package: String,
}

impl Piece for AptPackage {
    fn execute(&self) -> ExecutionResult {
        // Safety: we implement the method below and know it will only return Some
        Self::execute_bulk(&[self]).unwrap()
    }

    fn execute_bulk(pieces: &[&Self]) -> Option<ExecutionResult> {
        Some(Self::apt_command("install", pieces))
    }

    fn undo(&self) -> Option<ExecutionResult> {
        // Safety: we implement the method below and know it will only return Some
        Self::undo_bulk(&[self]).unwrap()
    }

    fn undo_bulk(pieces: &[&Self]) -> Option<Option<ExecutionResult>> {
        Some(Some(Self::apt_command("remove", pieces)))
    }
}

impl AptPackage {
    fn apt_command(command: &str, pieces: &[&Self]) -> ExecutionResult {
        process::Command::new("apt")
            .arg(command)
            .args(pieces.iter().map(|p| &p.package))
            .log_execution()
            .status()
            .to_execution_result()
    }
}
