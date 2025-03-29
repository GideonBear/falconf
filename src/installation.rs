use crate::machine::Machine;
use std::env::home_dir;
use std::{fs, io};
use std::path::PathBuf;
use crate::repo::Repo;

struct Installation {
    machine: Machine,
    repo: Repo,
}

impl Installation {
    fn get_root() -> Result<PathBuf, GetRootError> {
        Ok(home_dir()
            .ok_or(GetRootError::NoHomeDir)?
            .join(".falconf"))
    }
    
    fn new(remote: &str) -> Result<Self, InstallationCreateError> {
        let root = Self::get_root()?;
        if root.exists() {
            return Err(InstallationCreateError::Exists);
        }
        fs::create_dir(&root)?;
        
        let machine_path = root.join("machine");
        let repo_path = root.join("repo");
        
        let machine = Machine::new();
        fs::write(&machine_path, &machine.0)?;
        
        let repo = Repo::new(remote, &repo_path)?;
        
        Ok(Self {
            machine,
            repo,
        })
    }
    
    fn get() -> Result<Self, InstallationGetError> {
        let root = Self::get_root()?;

        if !root.is_dir() {
            return Err(InstallationGetError::NotFound);
        }

        let machine = Machine(
            fs::read_to_string(root.join("machine"))?
                .parse()
                .map_err(|_| {
                    InstallationGetError::InvalidInstallation("`machine` file does not contain a valid UUID".to_string())
                })?,
        );

        let repo = Repo::from_path(&root.join("repo"))?;

        Ok(Self { machine, repo })
    }
}

enum GetRootError {
    NoHomeDir,
}

enum InstallationGetError {
    NotFound,
    InvalidInstallation(String),
    GitError(git2::Error),
    Io(io::Error),
    NoHomeDir,
}

impl From<io::Error> for InstallationGetError {
    fn from(err: io::Error) -> Self {
        InstallationGetError::Io(err)
    }
}

impl From<git2::Error> for InstallationGetError {
    fn from(err: git2::Error) -> Self {
        InstallationGetError::GitError(err)
    }
}

impl From<GetRootError> for InstallationGetError {
    fn from(err: GetRootError) -> Self {
        match err {
            GetRootError::NoHomeDir => InstallationGetError::NoHomeDir,
        }
    }
}

enum InstallationCreateError {
    Exists,
    GitError(git2::Error),
    Io(io::Error),
    NoHomeDir,
}

impl From<io::Error> for InstallationCreateError {
    fn from(err: io::Error) -> Self {
        InstallationCreateError::Io(err)
    }
}

impl From<git2::Error> for InstallationCreateError {
    fn from(err: git2::Error) -> Self {
        InstallationCreateError::GitError(err)
    }
}

impl From<GetRootError> for InstallationCreateError {
    fn from(err: GetRootError) -> Self {
        match err {
            GetRootError::NoHomeDir => InstallationCreateError::NoHomeDir,
        }
    }
}
