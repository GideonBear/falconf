use crate::data::DataError;
use crate::machine::Machine;
use crate::repo::{Repo, RepoError};
use std::env::home_dir;
use std::path::PathBuf;
use std::{fs, io};

pub struct Installation {
    machine: Machine,
    repo: Repo,
}

impl Installation {
    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    pub fn repo(&mut self) -> &mut Repo {
        &mut self.repo
    }

    fn get_root() -> Result<PathBuf, GetRootError> {
        Ok(home_dir().ok_or(GetRootError::NoHomeDir)?.join(".falconf"))
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
        fs::write(&machine_path, machine.0)?;

        let repo = Repo::new(remote, &repo_path)?;

        Ok(Self { machine, repo })
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
                    InstallationGetError::InvalidInstallation(
                        "`machine` file does not contain a valid UUID".to_string(),
                    )
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
    Git(git2::Error),
    Data(DataError),
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
        InstallationGetError::Git(err)
    }
}

impl From<GetRootError> for InstallationGetError {
    fn from(err: GetRootError) -> Self {
        match err {
            GetRootError::NoHomeDir => InstallationGetError::NoHomeDir,
        }
    }
}

impl From<DataError> for InstallationGetError {
    fn from(err: DataError) -> Self {
        InstallationGetError::Data(err)
    }
}

impl From<RepoError> for InstallationGetError {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::Git(err) => err.into(),
            RepoError::Data(err) => err.into(),
            RepoError::InvalidInstallation(err) => InstallationGetError::InvalidInstallation(err),
        }
    }
}

enum InstallationCreateError {
    Exists,
    Git(git2::Error),
    Data(DataError),
    Io(io::Error),
    NoHomeDir,
    InvalidInstallation(String),
}

impl From<io::Error> for InstallationCreateError {
    fn from(err: io::Error) -> Self {
        InstallationCreateError::Io(err)
    }
}

impl From<git2::Error> for InstallationCreateError {
    fn from(err: git2::Error) -> Self {
        InstallationCreateError::Git(err)
    }
}

impl From<DataError> for InstallationCreateError {
    fn from(err: DataError) -> Self {
        InstallationCreateError::Data(err)
    }
}

impl From<GetRootError> for InstallationCreateError {
    fn from(err: GetRootError) -> Self {
        match err {
            GetRootError::NoHomeDir => InstallationCreateError::NoHomeDir,
        }
    }
}

impl From<RepoError> for InstallationCreateError {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::Git(err) => err.into(),
            RepoError::Data(err) => err.into(),
            RepoError::InvalidInstallation(err) => {
                InstallationCreateError::InvalidInstallation(err)
            }
        }
    }
}
