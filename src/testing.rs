use crate::logging::CommandExt;
use libc::{SIGTERM, kill};
use std::env::set_current_dir;
use std::io::BufRead;
use std::os::unix::prelude::CommandExt as UnixCommandExt;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread::sleep;
use tempdir::TempDir;

#[derive(Debug)]
struct TestRemote {
    repos_dir: TempDir,
    daemon: Child,
}

impl TestRemote {
    fn new() -> Self {
        // Create a temporary directory for the repositories
        // This will get cleaned up automatically when the TestRepository is dropped
        let repos_dir = TempDir::new("test_remote_repos").unwrap();
        let repo = repos_dir.path().join("test_repo.git");

        // Initialize the repository
        Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(&repo)
            .status_checked()
            .unwrap();

        // Start the git daemon
        let mut daemon = Command::new("git")
            .arg("daemon")
            .arg("--reuseaddr")
            .arg(format!(
                "--base-path={}",
                repos_dir.path().to_str().unwrap()
            ))
            .arg("--export-all")
            .arg("--enable=receive-pack")
            .arg("--verbose")
            .process_group(0) // See Drop implementation below
            .spawn()
            .unwrap();

        // Wait for the daemon to be ready
        sleep(std::time::Duration::from_secs(1));
        assert!(daemon.try_wait().unwrap().is_none());

        Self { repos_dir, daemon }
    }

    fn address(&self) -> &str {
        "git://localhost/test_repo.git"
    }

    fn clone_and_enter(&self) -> PathBuf {
        let tempdir = TempDir::new("test_local_repo").unwrap();
        let local = tempdir.path().join("test_repo");

        Command::new("git")
            .arg("clone")
            .arg(self.address())
            .arg(&local)
            .status_checked()
            .unwrap();

        set_current_dir(&local).unwrap();

        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("testing@example.com")
            .arg("--local")
            .status_checked()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .arg("--local")
            .status_checked()
            .unwrap();

        local
    }
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
    fn test_test_remote() {
        let mut remote = TestRemote::new();

        // 1

        let local_1 = remote.clone_and_enter();

        OpenOptions::new()
            .create(true)
            .write(true)
            .open(local_1.join("test_file"))
            .unwrap()
            .write_all(b"test")
            .unwrap();

        Command::new("git")
            .arg("add")
            .arg(".")
            .status_checked()
            .unwrap();

        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("test commit")
            .status_checked()
            .unwrap();

        Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("main")
            .status_checked()
            .unwrap();

        // 2

        let local_2 = remote.clone_and_enter();

        assert!(local_2.join("test_file").exists());
        assert_eq!(
            std::fs::read_to_string(local_2.join("test_file")).unwrap(),
            "test"
        );
    }
}
