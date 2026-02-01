use crate::cli::{PieceRef, TopLevelArgs};
use crate::installation::Installation;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, eyre};
use command_error::CommandExt;
use ctor::ctor;
use libc::{SIGTERM, kill};
use log::LevelFilter;
use std::env::set_current_dir;
use std::io::{BufRead, BufReader, Read};
use std::os::unix::prelude::CommandExt as UnixCommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{LazyLock, Mutex, MutexGuard};
use tempdir::TempDir;

static PORT_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

#[derive(Debug)]
pub struct TestRemote {
    _repos_dir: TempDir,
    daemon: Child,
    _port_mutex: MutexGuard<'static, ()>,
}

impl TestRemote {
    pub fn new() -> Result<Self> {
        // Wait until the port is available
        #[expect(
            clippy::missing_panics_doc,
            reason = "Panic in test is allowed, and cannot `?` here"
        )]
        let port_mutex = PORT_MUTEX.lock().unwrap();

        // Create a temporary directory for the repositories
        // This will get cleaned up automatically when the TestRepository is dropped
        let repos_dir = TempDir::new("test_remote_repos")?;
        let repo = repos_dir.path().join("test_repo.git");

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

        // Start the git daemon
        let mut daemon = Command::new("git")
            .stderr(Stdio::piped())
            .arg("daemon")
            .arg("--reuseaddr")
            .arg(format!(
                "--base-path={}",
                repos_dir
                    .path()
                    .to_str()
                    .ok_or_eyre("Invalid path (not unicode)")?
            ))
            .arg("--export-all")
            .arg("--enable=receive-pack")
            .arg("--verbose")
            .process_group(0) // See Drop implementation below
            .spawn_checked()?;

        // Wait for the daemon to be ready
        #[expect(clippy::missing_panics_doc, reason = "We captured stderr above")]
        wait_for_line(daemon.child_mut().stderr.as_mut().unwrap(), |l| {
            l.contains("Ready to rumble")
        })?;

        // This is not necessary
        // if daemon.try_wait_checked()?.is_some() {
        //     return Err(eyre!("git daemon died"));
        // }

        Ok(Self {
            _repos_dir: repos_dir,
            daemon: daemon.into_child(),
            _port_mutex: port_mutex,
        })
    }

    #[expect(clippy::unused_self)]
    pub fn address(&self) -> &'static str {
        "git://localhost/test_repo.git"
    }

    fn clone_and_enter(&self) -> Result<TempDirSub> {
        let tempdir = TempDir::new("test_local_repo")?;
        let local = tempdir.path().join("test_repo");

        Command::new("git")
            .arg("clone")
            .arg(self.address())
            .arg(&local)
            .status_checked()?;

        set_current_dir(&local)?;

        // This is normally handled by Repo::init but we do it manually here
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("testing@example.com")
            .arg("--local")
            .status_checked()?;
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .arg("--local")
            .status_checked()?;

        if !local.try_exists()? {
            return Err(eyre!("Local repo should exist after cloning"));
        }

        Ok(TempDirSub::new(tempdir, local))
    }
}

impl Drop for TestRemote {
    fn drop(&mut self) {
        // Necessary to kill all its children as well
        let pgid = i32::try_from(self.daemon.id()).unwrap();
        unsafe {
            kill(-pgid, SIGTERM);
        }
    }
}

fn wait_for_line<F: Fn(String) -> bool>(io: impl Read, cond: F) -> Result<()> {
    let reader = BufReader::new(io);

    for line in reader.lines() {
        let line = line?;
        if cond(line) {
            break;
        }
    }

    Ok(())
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

#[ctor]
fn setup_test() {
    color_eyre::install().unwrap();

    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;

    #[test]
    fn test_test_remote() -> Result<()> {
        let remote = TestRemote::new()?;

        // 1

        let local_1 = remote.clone_and_enter()?;

        assert!(local_1.path.try_exists()?); // Quick test to make sure the tempdir wasn't dropped

        OpenOptions::new()
            .create(true)
            // Not necessary here since the file doesn't exist yet, but this is what clippy demands
            .truncate(true)
            .write(true)
            .open(local_1.path.join("test_file"))?
            .write_all(b"test")?;

        Command::new("git").arg("add").arg(".").status_checked()?;

        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("test commit")
            .status_checked()?;

        Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("main")
            .status_checked()?;

        // 2

        let local_2 = remote.clone_and_enter()?;

        assert!(local_2.path.join("test_file").try_exists()?);
        assert_eq!(
            std::fs::read_to_string(local_2.path.join("test_file"))?,
            "test"
        );

        Ok(())
    }
}
