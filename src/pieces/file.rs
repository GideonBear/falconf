use crate::cli::AddArgs;
use crate::installation::Installation;
use crate::logging::CommandExt;
use crate::piece::Piece;
use color_eyre::eyre::{WrapErr, eyre};
use color_eyre::{Report, Result};
use serde::{Deserialize, Serialize};
use std::fs::remove_file;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    /// The location the file should be linked to
    location: PathBuf,
    // TODO
    // // TODO: Check if hardlinks actually work when new versions of files are pulled with git
    // /// If the file should be a hardlink or symlink
    // hardlink: bool,
    // TODO
    // // TODO: note that this comment is weird
    // /// If the file should be created as sudo
    // sudo: bool,
    /// The directory where the files are stored in the repo
    target_dir: PathBuf,
}

impl Piece for File {
    fn _execute(&self) -> Result<()> {
        // TODO: do stuff if the file exists

        let target_file = self.target_file()?;

        let mut cmd = Command::new("ln");
        cmd.arg(target_file).arg(&self.location);
        // if !self.hardlink {
        cmd.arg("--symbolic");
        // }

        cmd.status_checked()?;
        Ok(())
    }

    fn _undo(&self) -> Option<Result<()>> {
        Some(remove_file(&self.location).map_err(Report::from))
    }
}

impl File {
    fn target_file(&self) -> Result<PathBuf> {
        Ok(self.target_dir.join(
            self.location
                .strip_prefix("/")
                .wrap_err("Invalid file location (no leading slash). Unreachable, we checked for this at construction time.")?,
        ))
    }

    pub fn from_cli(args: AddArgs, installation: &Installation) -> Result<Self> {
        if args.value.len() != 1 {
            return Err(eyre!(
                "Expected a singular value (file location) for 'file' piece, got '{:?}'.",
                args.value
            ));
        }
        let location = args.value[0].clone();
        if !location.starts_with('/') {
            return Err(eyre!(
                "File location must be an absolute path (starting with '/'), got '{location:?}'."
            ));
        }
        let location = location.into();

        let target_dir = installation.repo().file_dir()?;

        Ok(Self {
            location,
            target_dir,
        })
    }
}
