use crate::cli::init::tests::init_util;
use crate::cli::{PieceRef, TopLevelArgs};
use crate::installation::Installation;
use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use command_error::CommandExt;
use ctor::ctor;
use log::LevelFilter;
use std::fs::{create_dir, rename};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[derive(Debug)]
pub struct TestRemote {
    _repo_dir: TempDir,
    pub address: String,
}

impl TestRemote {
    pub fn new() -> Result<Self> {
        // Create a temporary directory for the repositories
        // This will get cleaned up automatically when the TestRemote is dropped
        let repo_dir = TempDir::new()?;
        let repo = repo_dir.path().join("test_repo.git");

        // Initialize the repository
        Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(&repo)
            .status_checked()?;

        // Make sure the branch is called "main". Even though the client also has to set the branch,
        //  this still seems to be necessary. I really don't want to research why. This works.
        Command::new("git")
            .arg("-C")
            .arg(&repo)
            .arg("branch")
            .arg("-m")
            .arg("main")
            .status_checked()?;

        let address = format!(
            "file://{}",
            repo.to_str().ok_or_eyre("Invalid path (not unicode)")?
        );

        Ok(Self {
            _repo_dir: repo_dir,
            address,
        })
    }
}

/// A subpath of a `TempDir` that owns the `TempDir` so it drops only when the `TempDirSub` is dropped
pub struct TempDirSub {
    path: PathBuf,
    _tempdir: TempDir,
}

impl TempDirSub {
    pub fn new(tempdir: TempDir, path: PathBuf) -> Self {
        Self {
            path,
            _tempdir: tempdir,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

pub fn get_piece(falconf_dir: &Path, position: usize) -> Result<PieceRef> {
    let top_level_args = TopLevelArgs::new_testing(falconf_dir.to_path_buf(), true);
    let mut installation = Installation::get(&top_level_args)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    let (&id, _piece) = pieces
        .get_index(position)
        .ok_or_eyre("No piece at that index")?;

    Ok(PieceRef::Id(id))
}

/// Simulates one remote and several machines, sharing one filesystem location.
/// Only the entered machine's files are at `fs`, the others are stashed away.
/// All machines are initialized, machine 0 is newly initialized.
pub struct TestEnv {
    pub remote: TestRemote,
    pub fs: PathBuf,
    stashes: Vec<PathBuf>,
    falconf_dirs: Vec<TempDirSub>,
    _tempdir: TempDir,
}

impl TestEnv {
    pub fn new(count: usize) -> Result<Self> {
        let remote = TestRemote::new()?;
        let tempdir = TempDir::new()?;
        let (stashes, falconf_dirs) = (0..count)
            .map(|i| {
                let name = format!("machine_{i}");
                let stash = tempdir.path().join(&name);
                create_dir(&stash)?;
                let falconf_dir = init_util(&remote, i == 0, name)?;
                Ok((stash, falconf_dir))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .unzip();
        Ok(Self {
            remote,
            fs: tempdir.path().join("fs"),
            stashes,
            falconf_dirs,
            _tempdir: tempdir,
        })
    }

    pub fn with(&mut self, machine: usize, f: impl FnOnce(&PathBuf) -> Result<()>) -> Result<()> {
        rename(&self.stashes[machine], &self.fs)?;
        let result = f(self.falconf_dirs[machine].path());
        rename(&self.fs, &self.stashes[machine])?;
        result
    }
}

#[ctor(unsafe)]
fn setup_test() {
    color_eyre::install().unwrap();

    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();
}
