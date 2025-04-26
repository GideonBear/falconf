use crate::data::Data;
use crate::machine::{Machine, MachineData};
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, eyre};
use git2::Repository;
use std::fs::File;
use std::path::{Path, PathBuf};

const BRANCH: &str = "main";

pub struct Repo {
    repository: Repository,
    data: Data,
}

// TODO: instead of write_and_push, make function that takes in a closure gets a &mut Data?
//  After the closure, write it. This makes sure that we never forget to write.
//  see iso-updater

impl Repo {
    pub fn init(
        remote: &str,
        path: &Path,
        machine: Machine,
        machine_data: MachineData,
        new: bool,
    ) -> Result<()> {
        let repository = Repository::clone(remote, path)?;

        let mut repo = if new {
            let data = Data::init_new();
            let repo = Self { repository, data };

            File::create(repo.file_dir()?.join(".gitkeep"))?;

            // We push below
            repo
        } else {
            Self::from_repository(repository)?
        };

        let data = repo.data_mut();
        data.machines_mut().insert(machine, machine_data);
        repo.write_and_push()?;
        Ok(())
    }

    pub fn file_dir(&self) -> Result<PathBuf> {
        Ok(self
            .repository
            .workdir()
            .ok_or_eyre("Repository is bare")?
            .join("files"))
    }

    pub fn data_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    pub fn get_from_path(path: &Path) -> Result<Self> {
        Repository::open(path).map(Repo::from_repository)?
    }

    fn get_data(repository: &Repository) -> Result<Data> {
        Data::from_file(&data_path_from_repository(repository)?)
    }

    fn update_data(&mut self) -> Result<()> {
        self.data = Self::get_data(&self.repository)?;
        Ok(())
    }

    fn from_repository(repository: Repository) -> Result<Self> {
        let data = Self::get_data(&repository)?;
        Ok(Self { repository, data })
    }

    fn pull(&self) -> Result<()> {
        let mut remote = self.repository.find_remote("origin")?;
        remote.fetch(&[BRANCH], None, None)?;
        let fetch_head = self.repository.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repository.reference_to_annotated_commit(&fetch_head)?;
        let analysis = self.repository.merge_analysis(&[&fetch_commit])?;
        if analysis.0.is_up_to_date() {
            Ok(())
        } else if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{BRANCH}");
            let mut reference = self.repository.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            self.repository.set_head(&refname)?;
            self.repository
                .checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            Ok(())
        } else {
            Err(eyre!("Branches have diverted"))
        }
    }

    fn write_data(&self) -> Result<()> {
        self.data
            .to_file(&data_path_from_repository(&self.repository)?)?;
        Ok(())
    }

    fn commit(&self) -> Result<(), git2::Error> {
        let mut index = self.repository.index()?;

        index.add_all(["."], git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let oid = index.write_tree()?;
        let signature = self.repository.signature()?;
        let tree = self.repository.find_tree(oid)?;

        let message = "Falconf update";

        self.repository
            .commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?;

        Ok(())
    }

    fn push(&self) -> Result<()> {
        // TODO: check for failed push
        //  > Note that you’ll likely want to use RemoteCallbacks and set push_update_reference
        //  > to test whether all the references were pushed successfully.
        //  And return PushPullError::divergence when it fails because of divergence in the remote
        let mut remote = self.repository.find_remote("origin")?;
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
