use color_eyre::Result;
use color_eyre::eyre::{OptionExt, eyre};
use command_error::CommandExt;
use libc::{SIGTERM, kill};
use std::env::set_current_dir;
use std::os::unix::prelude::CommandExt as UnixCommandExt;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::thread::sleep;
use tempdir::TempDir;

static PORT_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

#[derive(Debug)]
pub struct TestRemote {
    repos_dir: TempDir,
    daemon: Child,
    port_mutex: MutexGuard<'static, ()>,
}

impl TestRemote {
    pub fn new() -> Result<Self> {
        // Wait until the port is available
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

        // TODO: is this necessary?
        // Make sure the branch is called "main"
        Command::new("git")
            .arg("-C")
            .arg(&repo)
            .arg("branch")
            .arg("-m")
            .arg("main")
            .status_checked()?;

        // Start the git daemon
        let mut daemon = Command::new("git")
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
            .spawn()?;

        // Wait for the daemon to be ready
        sleep(std::time::Duration::from_millis(250));
        assert!(daemon.try_wait()?.is_none());

        Ok(Self {
            repos_dir,
            daemon,
            port_mutex,
        })
    }

    pub fn address(&self) -> &str {
        "git://localhost/test_repo.git"
    }

    fn clone_and_enter(&self) -> Result<LocalRepo> {
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

        // We need to return a struct to make sure the TempDir isn't dropped at the end of this function
        Ok(LocalRepo {
            path: local,
            _tempdir: tempdir,
        })
    }
}

pub struct LocalRepo {
    path: PathBuf,
    _tempdir: TempDir,
}

impl Drop for TestRemote {
    fn drop(&mut self) {
        // Necessary to kill all its children as well
        let pgid = self.daemon.id() as i32;
        unsafe {
            kill(-pgid, SIGTERM);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;

    #[test]
    fn test_test_remote() -> Result<()> {
        color_eyre::install().ok();

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
