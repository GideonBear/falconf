use crate::logging::CommandExt;
use crate::piece::ResultExitStatusExt;
use crate::piece::{ExecutionError, ExecutionResult, Piece};
use crate::repo::find_file;
use crate::utils;
use serde::{Deserialize, Serialize};
use std::fs::remove_file;
use std::path::PathBuf;

/// Sym/hardlink a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    /// The location the file should be linked to
    location: PathBuf,
    // TODO: Check if hardlinks actually work when new versions of files are pulled with git
    /// If the file should be a hardlink or symlink
    hardlink: bool,
    /// If the file should be created as sudo
    sudo: bool,
}

impl Piece for File {
    fn execute(&self) -> ExecutionResult {
        let repo_file = find_file(&self.location);

        let mut cmd = utils::if_sudo("ln", self.sudo);
        cmd.arg(repo_file).arg(&self.location);
        if !self.hardlink {
            cmd.arg("--symbolic");
        }

        cmd.log_execution()?.status().to_execution_result()
    }

    fn undo(&self) -> Option<ExecutionResult> {
        Some(remove_file(&self.location).map_err(ExecutionError::from))
    }
}
