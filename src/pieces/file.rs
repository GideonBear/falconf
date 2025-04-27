use crate::cli::AddArgs;
use crate::installation::Installation;
use crate::logging::CommandExt;
use crate::piece::Piece;
use crate::utils::confirm;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, WrapErr, eyre};
use log::info;
use serde::{Deserialize, Serialize};
use std::fs::remove_file;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    /// The location the file should be linked to
    location: PathBuf,
    // TODO
    // // TODO: Check if hardlinks actually work when new versions of files are pulled with git.
    // //  See the Trello thing
    // /// If the file should be a hardlink or symlink
    // hardlink: bool,
    // TODO
    // // TODO: note that this comment is weird
    // /// If the file should be created as sudo
    // sudo: bool,
    /// The directory where the files are stored in the repository
    target_dir: PathBuf,
    /// What the file should look like before the operation if it exists
    expected_previous_content: Option<String>,
}

impl Piece for File {
    fn _execute(&self) -> Result<()> {
        let target_file = self.target_file()?;

        if self.location.exists() {
            if self.location.is_symlink() {
                return Err(eyre!("File already exists and is a symlink."));
            }

            if let Some(expected_previous_content) = &self.expected_previous_content {
                let actual_content = std::fs::read_to_string(&self.location)?;
                if actual_content != *expected_previous_content {
                    return Err(eyre!(
                        "File already exists and has different content than expected. Expected content: '{expected_previous_content}', actual content: '{actual_content}'."
                    ));
                }
                info!("File already exists but has expected content; overwriting.");
            } else {
                let diff = Command::new("diff")
                    .arg(&target_file)
                    .arg(&self.location)
                    .output_fallible()?;

                #[allow(clippy::collapsible_else_if)] // Clearer this way
                if diff.status.success() {
                    info!("File already exists but is identical; overwriting.");
                } else {
                    if confirm(&format!(
                        "File already exists and is different. Diff:\n{}\nConsider adding an expected content string to the file to prevent this from happening in the future.\nDo you want to overwrite the file?",
                        String::from_utf8_lossy(&diff.stdout)
                    ))? {
                        info!("Overwriting file according to user input.");
                    } else {
                        return Err(eyre!("Aborted."));
                    }
                }
            }
        } else if let Some(expected_previous_content) = &self.expected_previous_content {
            return Err(eyre!(
                "File was expected to exist and have content, but it doesn't exist. Expected content: '{expected_previous_content}'."
            ));
        }

        let parent = target_file
            .parent()
            .ok_or_eyre("File doesn't have parent")?;
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }

        Command::new("ln")
            .arg(target_file)
            .arg(&self.location)
            .arg("--symbolic")
            .status_checked()?;
        Ok(())
    }

    fn _undo(&self) -> Option<Result<()>> {
        Some(remove_file(&self.location).wrap_err("Failed to remove file as part of undo"))
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
            expected_previous_content: None,
        })
    }
}
