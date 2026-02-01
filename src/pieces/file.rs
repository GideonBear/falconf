use crate::cli::AddArgs;
use crate::execution_data::ExecutionData;
use crate::logging::CommandExt;
use crate::piece::NonBulkPiece;
use crate::utils::{confirm, create_parent};
use color_eyre::Result;
use color_eyre::eyre::{WrapErr, eyre};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs::{remove_file, rename};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    /// The location the file should be linked to
    location: PathBuf,
    // TODO(med): Add hardlink support
    //  Check if hardlinks actually work when new versions of files are pulled with git.
    //  See the Trello thing
    // /// If the file should be a hardlink or symlink
    // hardlink: bool,
    // TODO(med): Add sudo support
    // /// If the file should be created as root
    // sudo: bool,
    /// What the file should look like before the operation if it exists
    expected_previous_content: Option<String>,
}

impl NonBulkPiece for File {
    fn execute(&mut self, execution_data: &ExecutionData) -> Result<()> {
        let target_file = self.target_file(execution_data);

        if !target_file.exists() {
            info!("Repo (target) file doesn't exist, assuming this is newly added");
            debug!(
                "Moving the file into the repo: {} to {}",
                self.location.display(),
                target_file.display()
            );
            create_parent(&target_file)?;
            rename(&self.location, &target_file).wrap_err("Failed to move file into repo")?;
        }

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
                        "File already exists and is different. Diff between the repo content and actual content:\n{}\nConsider adding an expected content string to the file to prevent this from happening in the future.\nDo you want to overwrite the file?",
                        String::from_utf8_lossy(&diff.stdout)
                    ))? {
                        info!("Overwriting file according to user input.");
                        debug!("Removing file");
                        remove_file(&self.location).wrap_err("Failed to remove file.")?;
                    } else {
                        return Err(eyre!("Aborted"));
                    }
                }
            }
        } else if let Some(expected_previous_content) = &self.expected_previous_content {
            return Err(eyre!(
                "File was expected to exist and have content, but it doesn't exist. Expected content: '{expected_previous_content}'."
            ));
        }

        create_parent(&self.location)?;

        Command::new("ln")
            .arg(target_file)
            .arg(&self.location)
            .arg("--symbolic")
            .status_checked()?;
        Ok(())
    }

    fn undo(&mut self, _execution_data: &ExecutionData) -> Result<()> {
        if !self.location.is_symlink() {
            return Err(eyre!("File is not a symlink."));
        }
        remove_file(&self.location).wrap_err("Failed to remove file as part of undo")
    }
}

impl File {
    /// Return the file's location in the file dir; the target of the symlink
    fn target_file(&self, execution_data: &ExecutionData) -> PathBuf {
        execution_data.file_dir.join(self.relative_location())
    }

    /// Return the file's location relative to /; the target of the symlink relative to the file dir
    pub fn relative_location(&self) -> &Path {
        #[expect(clippy::missing_panics_doc, reason = "illegal configuration")]
        self.location
            .strip_prefix("/").expect("Invalid file location (no leading slash). Unreachable, we checked for this at construction time.")
    }

    pub fn from_cli(args: &AddArgs) -> Result<Self> {
        if args.value.len() != 1 {
            return Err(eyre!(
                "Expected a singular value (file location) for 'file' piece, got '{:?}'.",
                args.value
            ));
        }

        let location = args.value[0].clone();
        // TODO(low): This does resolve symlinks, is that okay?
        let location = std::fs::canonicalize(&location).wrap_err_with(|| {
            format!("Failed to canonicalize file '{location}'. Does it exist?")
        })?;

        if !location.starts_with(PathBuf::from("/")) {
            return Err(eyre!(
                "File location must be an absolute path (starting with '/'), got '{location:?}'. Unreachable because we just canonicalized it."
            ));
        }

        Ok(Self {
            location,
            expected_previous_content: None,
        })
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tracking file at: {}", self.location.display())
    }
}

// File is mostly tested in sync
#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::cli::TopLevelArgs;
    use crate::cli::add::tests::add_util_no_test_run;
    use crate::cli::init::tests::init_util;
    use crate::cli::sync::{SyncArgs, sync};
    use crate::testing::TestRemote;
    use color_eyre::eyre::OptionExt;
    use std::fs;
    use std::fs::{create_dir, remove_dir_all};
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn test_file_dir() -> Result<()> {
        let remote = TestRemote::new()?;
        let temp = TempDir::new("test_falconf_files")?;
        let test_d = temp.path().join("dir");
        create_dir(&test_d)?;
        let test_1 = test_d.join("test_1");
        fs::File::create(&test_1)?.write_all(b"test_1_content")?;
        let test_2 = test_d.join("test_2");
        fs::File::create(&test_2)?.write_all(b"test_2_content")?;

        let local_1 = init_util(&remote, true)?;
        let test_d_s = test_d.to_str().ok_or_eyre("Invalid path")?.to_string();
        add_util_no_test_run(local_1.path(), crate::cli::add::Piece::File, vec![test_d_s])?;

        assert!(test_1.exists());
        assert_eq!(fs::read_to_string(&test_1)?, "test_1_content");
        assert!(test_2.exists());
        assert_eq!(fs::read_to_string(&test_2)?, "test_2_content");

        // Switching to being another machine, the dir doesn't exist yet
        remove_dir_all(&test_d)?;
        assert!(!test_d.exists());
        assert!(!test_1.exists());
        assert!(!test_2.exists());

        let local_2 = init_util(&remote, false)?;

        let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), false);
        let args = SyncArgs {};
        sync(top_level_args, args)?;

        // After syncing, the dir is created
        assert!(test_1.exists());
        assert_eq!(fs::read_to_string(&test_1)?, "test_1_content");
        assert!(test_2.exists());
        assert_eq!(fs::read_to_string(&test_2)?, "test_2_content");

        Ok(())
    }
}
