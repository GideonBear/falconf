use crate::cli::add;
use crate::cli::add::tests::add_util_no_test_run;
use crate::cli::init::tests::init_util;
use crate::cli::sync::tests::sync_util;
use crate::cli::{PieceRef, TopLevelArgs};
use crate::installation::Installation;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, eyre};
use command_error::CommandExt;
use ctor::ctor;
use log::LevelFilter;
use std::fs;
use std::fs::{create_dir, rename};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::Duration;
use tempfile::TempDir;

#[derive(Debug)]
pub struct TestRemote {
    _repo_dir: TempDir,
    /// The bare repository
    pub path: PathBuf,
    pub address: String,
    daemon: Option<Child>,
}

impl TestRemote {
    /// A remote accessed with `file://`
    pub fn new() -> Result<Self> {
        let (repo_dir, path) = Self::init_bare()?;
        let address = format!(
            "file://{}",
            path.to_str().ok_or_eyre("Invalid path (not unicode)")?
        );

        Ok(Self {
            _repo_dir: repo_dir,
            path,
            address,
            daemon: None,
        })
    }

    /// A remote served by `git daemon`, which (unlike `file://`) runs the remote's hooks
    pub fn new_daemon() -> Result<Self> {
        let (repo_dir, path) = Self::init_bare()?;
        let port = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
        let exec_path = Command::new("git")
            .arg("--exec-path")
            .output_checked_utf8()?
            .stdout;
        // Not `git daemon`, which runs `git-daemon` as a child that wouldn't be killed on drop
        let daemon = Command::new(Path::new(exec_path.trim()).join("git-daemon"))
            .arg("--listen=127.0.0.1")
            .arg(format!("--port={port}"))
            .arg(format!("--base-path={}", repo_dir.path().display()))
            .args(["--export-all", "--enable=receive-pack", "--reuseaddr"])
            .arg(repo_dir.path())
            // So the user's config (e.g. `core.hooksPath`) can't stop the remote's hooks from running
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .spawn()?;
        let remote = Self {
            _repo_dir: repo_dir,
            path,
            address: format!("git://127.0.0.1:{port}/test_repo.git"),
            daemon: Some(daemon),
        };

        for _ in 0..100 {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return Ok(remote);
            }
            sleep(Duration::from_millis(50));
        }
        Err(eyre!("git daemon didn't start listening"))
    }

    fn init_bare() -> Result<(TempDir, PathBuf)> {
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

        Ok((repo_dir, repo))
    }
}

impl Drop for TestRemote {
    fn drop(&mut self) {
        if let Some(daemon) = &mut self.daemon {
            let _ = daemon.kill();
            #[expect(
                clippy::disallowed_methods,
                reason = "It was killed, so its exit status is irrelevant"
            )]
            let _ = daemon.wait();
        }
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
        Self::with_remote(TestRemote::new()?, count)
    }

    pub fn with_remote(remote: TestRemote, count: usize) -> Result<Self> {
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

/// Returns the output of `git status --porcelain` in the repository of `f`
pub fn git_status(f: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(f.join("repository"))
        .args(["status", "--porcelain"])
        .output_checked_utf8()?;
    Ok(output.stdout)
}

/// Two machines with two files, `a` and `b`, synced to both
pub fn setup_two_files() -> Result<(TestEnv, PathBuf, PathBuf)> {
    let mut env = TestEnv::new(2)?;
    let a = env.fs.join("a.txt");
    let b = env.fs.join("b.txt");

    env.with(0, |f| {
        for (file, content) in [(&a, "a"), (&b, "b")] {
            fs::write(file, content)?;
            let file_s = file.to_str().ok_or_eyre("Invalid path")?.to_string();
            add_util_no_test_run(f, add::Piece::File, vec![file_s])?;
        }
        Ok(())
    })?;
    env.with(1, |f| sync_util(f))?;

    Ok((env, a, b))
}

#[ctor(unsafe)]
fn setup_test() {
    color_eyre::install().unwrap();

    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();
}
