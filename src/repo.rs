use std::path::{Path, PathBuf};
use git2::Repository;

pub struct Repo {
    // TODO
    repo: Repository,
}

impl Repo {
    pub fn new(remote: &str, path: &Path) -> Result<Self, git2::Error> {
        Repository::clone(remote, path).map(|x| x.into())
    }
    
    pub fn from_path(path: &Path) -> Result<Self, git2::Error> {
        Repository::open(path).map(|x| x.into())
    }

    pub fn file(location: &Path) -> PathBuf {
        todo!();
    }
}

impl From<Repository> for Repo {
    fn from(repo: Repository) -> Self {
        // TODO
        Self { repo }
    }
}
