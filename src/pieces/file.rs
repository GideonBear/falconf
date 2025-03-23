use std::fs::remove_file;
use std::path::PathBuf;
use crate::piece::{ExecutionError, ExecutionResult, Piece, ResultExitStatusExt};
use crate::utils;

/// Sym/hardlink a file
struct File {
    location: PathBuf,
    /// If the file should be a hardlink or symlink
    hardlink: bool,
    /// If the file should be created as sudo
    sudo: bool,
}

impl Piece for File {
    fn execute(&self) -> ExecutionResult {
        let repo_file = find_file(&self.location);

        let mut cmd = utils::if_sudo("ln", self.sudo)
            .arg(&repo_file)
            .arg(&self.location);
        if !self.hardlink {
            cmd.arg("--symbolic");
        }

        cmd
            .status()
            .to_execution_result()
    }

    fn undo(&self) -> Option<ExecutionResult> {
        Some(
            remove_file(&self.location)
                .map_err(|e| ExecutionError::from(e))
        )
    }
}