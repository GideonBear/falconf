use std::path::{Path, PathBuf};
use git2::Repository;
use crate::data::{Data, DataError};

pub struct Repo {
    repo: Repository,
    data: Data,
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
        let data = Data::from_file(&repo.workdir().ok_or(RepoError::InvalidInstallation("Git repository is bare".to_string()))?.join("data.json"))?;
        Ok(Self { repo, data })
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
