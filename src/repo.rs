use crate::data::{Data, DataError};
use git2::Repository;
use std::path::{Path, PathBuf};

pub struct Repo {
    repo: Repository,
    pub data: Data,
}

impl Repo {
    pub fn new(remote: &str, path: &Path) -> Result<Self, RepoError> {
        Repository::clone(remote, path).map(Repo::from_repository)?
    }

    pub fn from_path(path: &Path) -> Result<Self, RepoError> {
        Repository::open(path).map(Repo::from_repository)?
    }

    pub fn file(location: &Path) -> PathBuf {
        todo!();
    }

    pub fn from_repository(repo: Repository) -> Result<Self, RepoError> {
        let data = Data::from_file(&data_path_from_repository(&repo)?)?;
        Ok(Self { repo, data })
    }

    pub fn write_data(&self) -> Result<(), RepoError> {
        self.data.to_file(&data_path_from_repository(&self.repo)?)?;
        Ok(())
    }
}

pub enum RepoError {
    InvalidInstallation(String),
    Git(git2::Error),
    Data(DataError),
}

impl From<git2::Error> for RepoError {
    fn from(err: git2::Error) -> Self {
        RepoError::Git(err)
    }
}

impl From<DataError> for RepoError {
    fn from(err: DataError) -> Self {
        RepoError::Data(err)
    }
}

fn data_path_from_repository(repo: &Repository) -> Result<PathBuf, RepoError> {
    Ok(repo
        .workdir()
        .ok_or(RepoError::InvalidInstallation(
            "Git repository is bare".to_string(),
        ))?
        .join("data.json"))
}
