use crate::data::{Data, DataError};
use git2::Repository;
use std::path::{Path, PathBuf};

const BRANCH: &str = "main";

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

    fn get_data(repo: &Repository) -> Result<Data, RepoError> {
        Ok(Data::from_file(&data_path_from_repository(repo)?)?)
    }

    fn update_data(&mut self) -> Result<(), RepoError> {
        self.data = Self::get_data(&self.repo)?;
        Ok(())
    }

    fn from_repository(repo: Repository) -> Result<Self, RepoError> {
        let data = Self::get_data(&repo)?;
        Ok(Self { repo, data })
    }

    fn pull(&self) -> Result<(), PushPullError> {
        let mut remote = self.repo.find_remote("origin")?;
        remote.fetch(&[BRANCH], None, None)?;
        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = self.repo.merge_analysis(&[&fetch_commit])?;
        if analysis.0.is_up_to_date() {
            Ok(())
        } else if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", BRANCH);
            let mut reference = self.repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            self.repo.set_head(&refname)?;
            self.repo
                .checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            Ok(())
        } else {
            Err(PushPullError::Divergence)
        }
    }

    fn write_data(&self) -> Result<(), RepoError> {
        self.data.to_file(&data_path_from_repository(&self.repo)?)?;
        Ok(())
    }

    fn commit(&self) -> Result<(), git2::Error> {
        let mut index = self.repo.index()?;

        index.add_all(["."], git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let oid = index.write_tree()?;
        let signature = self.repo.signature()?;
        let tree = self.repo.find_tree(oid)?;

        let message = "Falconf update";

        self.repo
            .commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?;

        Ok(())
    }

    fn push(&self) -> Result<(), PushPullError> {
        // TODO: check for failed push
        //  > Note that you’ll likely want to use RemoteCallbacks and set push_update_reference
        //  > to test whether all the references were pushed successfully.
        //  And return PushPullError::divergence when it fails because of divergence in the remote
        let mut remote = self.repo.find_remote("origin")?;
        remote.push(&[BRANCH], None)?;
        Ok(())
    }

    pub fn pull_and_read(&mut self) -> Result<(), PushPullError> {
        self.pull()?;
        self.update_data()?;
        Ok(())
    }

    pub fn write_and_push(&self) -> Result<(), PushPullError> {
        self.write_data()?;
        self.commit()?;
        self.push()?;
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

enum PushPullError {
    Divergence,
    Git(git2::Error),
    Repo(RepoError),
}

impl From<git2::Error> for PushPullError {
    fn from(err: git2::Error) -> Self {
        PushPullError::Git(err)
    }
}

impl From<RepoError> for PushPullError {
    fn from(err: RepoError) -> Self {
        PushPullError::Repo(err)
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
