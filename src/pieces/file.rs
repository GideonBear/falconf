use crate::logging::CommandExt;
use crate::piece::Piece;
use crate::utils;
use color_eyre::eyre::WrapErr;
use color_eyre::{Report, Result};
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
    /// The directory where the files are stored in the repo
    target_dir: PathBuf,
}

impl Piece for File {
    fn execute(&self) -> Result<()> {
        let target_file = self.target_file()?;

        let mut cmd = utils::if_sudo("ln", self.sudo);
        cmd.arg(target_file).arg(&self.location);
        if !self.hardlink {
            cmd.arg("--symbolic");
        }

        cmd.status_checked()?;
        Ok(())
    }

    fn undo(&self) -> Option<Result<()>> {
        Some(remove_file(&self.location).map_err(Report::from))
    }
}

impl File {
    fn target_file(&self) -> Result<PathBuf> {
        Ok(self.target_dir.join(
            self.location
                .strip_prefix("/")
                .wrap_err("Invalid file location (no leading slash)")?,
        ))
    }
}
