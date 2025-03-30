use crate::logging::CommandExt;
use crate::piece::ResultExitStatusExt;
use crate::piece::{ExecutionResult, Piece};
use serde::{Deserialize, Serialize};
use std::process;

/// Install a package with apt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptPackage {
    /// The package to install
    package: String,
}

impl Piece for AptPackage {
    fn execute(&self) -> ExecutionResult {
        // Since execute_bulk is implemented we assume this is never called.
        panic!();
        Self::execute_bulk(&[self])
    }

    fn execute_bulk(pieces: &[&Self]) -> ExecutionResult {
        Self::apt_command("install", pieces)
    }

    fn undo(&self) -> Option<ExecutionResult> {
        // Since execute_bulk is implemented we assume this is never called.
        panic!();
        Some(Self::undo_bulk(&[self]))
    }

    fn undo_bulk(pieces: &[&Self]) -> ExecutionResult {
        Self::apt_command("remove", pieces)
    }
}

impl AptPackage {
    fn apt_command(command: &str, pieces: &[&Self]) -> ExecutionResult {
        process::Command::new("apt")
            .arg(command)
            .args(pieces.iter().map(|p| &p.package))
            .log_execution()?
            .status()
            .to_execution_result()
    }
}
