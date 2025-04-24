use crate::data::Data;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, eyre};
use git2::Repository;
use std::path::{Path, PathBuf};

const BRANCH: &str = "main";

pub struct Repo {
    repo: Repository,
    data: Data,
}

impl Repo {
    pub fn new(remote: &str, path: &Path) -> Result<Self> {
        Repository::clone(remote, path).map(Repo::from_repository)?
    }

    // TODO: pub fn create

    pub fn file_dir(&self) -> Result<PathBuf> {
        Ok(self
            .repo
            .workdir()
            .ok_or_eyre("Repo is bare")?
            .join("files"))
    }

    pub fn data_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        Repository::open(path).map(Repo::from_repository)?
    }

    fn get_data(repo: &Repository) -> Result<Data> {
        Data::from_file(&data_path_from_repository(repo)?)
    }

    fn update_data(&mut self) -> Result<()> {
        self.data = Self::get_data(&self.repo)?;
        Ok(())
    }

    fn from_repository(repo: Repository) -> Result<Self> {
        let data = Self::get_data(&repo)?;
        Ok(Self { repo, data })
    }

    fn pull(&self) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        remote.fetch(&[BRANCH], None, None)?;
        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = self.repo.merge_analysis(&[&fetch_commit])?;
        if analysis.0.is_up_to_date() {
            Ok(())
        } else if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{BRANCH}");
            let mut reference = self.repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            self.repo.set_head(&refname)?;
            self.repo
                .checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            Ok(())
        } else {
            Err(eyre!("Branches have diverted"))
        }
    }

    fn write_data(&self) -> Result<()> {
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

    fn push(&self) -> Result<()> {
        // TODO: check for failed push
        //  > Note that you’ll likely want to use RemoteCallbacks and set push_update_reference
        //  > to test whether all the references were pushed successfully.
        //  And return PushPullError::divergence when it fails because of divergence in the remote
        let mut remote = self.repo.find_remote("origin")?;
        remote.push(&[BRANCH], None)?;
        Ok(())
    }

    pub fn pull_and_read(&mut self) -> Result<()> {
        self.pull()?;
        self.update_data()?;
        Ok(())
    }

    pub fn write_and_push(&self) -> Result<()> {
        self.write_data()?;
        self.commit()?;
        self.push()?;
        Ok(())
    }
}

fn data_path_from_repository(repo: &Repository) -> Result<PathBuf> {
    Ok(repo
        .workdir()
        .ok_or_eyre("Git repository is bare")?
        .join("data.json"))
}
