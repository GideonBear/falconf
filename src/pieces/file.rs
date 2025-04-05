use crate::logging::CommandExt;
use crate::piece::ResultExitStatusExt;
use crate::piece::{ExecutionError, ExecutionResult, Piece};
use crate::utils;
use serde::{Deserialize, Serialize};
use std::fs::remove_file;
use std::path::{PathBuf, StripPrefixError};

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
    /// The directory where the files are stored in the repo
    target_dir: PathBuf,
}

impl Piece for File {
    fn execute(&self) -> ExecutionResult {
        let target_file = self.target_file()?;

        let mut cmd = utils::if_sudo("ln", self.sudo);
        cmd.arg(target_file).arg(&self.location);
        if !self.hardlink {
            cmd.arg("--symbolic");
        }

        cmd.log_execution()?.status().to_execution_result()
    }

    fn undo(&self) -> Option<ExecutionResult> {
        Some(remove_file(&self.location).map_err(ExecutionError::from))
    }
}

impl File {
    fn target_file(&self) -> Result<PathBuf, FileError> {
        Ok(self.target_dir.join(self.location.strip_prefix("/")?))
    }
}

#[derive(Debug, Clone)]
pub enum FileError {
    InvalidLocation,
}

impl From<StripPrefixError> for FileError {
    fn from(_: StripPrefixError) -> Self {
        FileError::InvalidLocation
    }
}
