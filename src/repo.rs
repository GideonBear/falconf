use crate::data::Data;
use crate::machine::{Machine, MachineData};
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, WrapErr, eyre};
use git2::Repository;
use log::debug;
use std::fs::{File, create_dir};
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
        debug!("Cloning repo");
        let repository = Repository::clone(remote, path).wrap_err("Failed to clone repository")?;

        let mut repo = if new {
            debug!("New, so initializing new");
            let data = Data::init_new();
            let repo = Self { repository, data };

            let file_dir = repo.file_dir().wrap_err("Failed to get file dir")?;
            create_dir(&file_dir).wrap_err("Failed to create file dir")?;
            File::create(file_dir.join(".gitkeep")).wrap_err("Failed to create .gitkeep")?;

            // We push below
            repo
        } else {
            Self::from_repository(repository).wrap_err("Failed to construct repo")?
        };

        let mut config = repo.repository.config().wrap_err("Failed to get config")?;
        config
            .set_str("user.name", "falconf")
            .wrap_err("Failed to set user.name")?;
        config
            .set_str("user.email", "falconf@example.com")
            .wrap_err("Failed to set user.email")?;

        let data = repo.data_mut();
        data.machines_mut().insert(machine, machine_data);
        repo.write_and_push().wrap_err("Failed to write_and_push")?;
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
        let repository = Repository::open(path).wrap_err("Failed to open repository")?;
        Self::from_repository(repository)
    }

    fn get_data(repository: &Repository) -> Result<Data> {
        Data::from_file(&data_path_from_repository(repository)?)
    }

    fn update_data(&mut self) -> Result<()> {
        self.data = Self::get_data(&self.repository).wrap_err("Failed to get data")?;
        Ok(())
    }

    fn from_repository(repository: Repository) -> Result<Self> {
        let data = Self::get_data(&repository).wrap_err("Failed to get data")?;
        Ok(Self { repository, data })
    }

    fn pull(&self) -> Result<()> {
        let mut remote = self
            .repository
            .find_remote("origin")
            .wrap_err("Failed to find remote")?;
        remote
            .fetch(&[BRANCH], None, None)
            .wrap_err("Failed to fetch")?;
        let fetch_head = self
            .repository
            .find_reference("FETCH_HEAD")
            .wrap_err("Failed to find fetch head")?;
        let fetch_commit = self
            .repository
            .reference_to_annotated_commit(&fetch_head)
            .wrap_err("Failed to convert fetch head to annotated commit")?;
        let analysis = self
            .repository
            .merge_analysis(&[&fetch_commit])
            .wrap_err("Failed to do merge analysis")?;
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

    fn commit(&self) -> Result<()> {
        let mut index = self.repository.index().wrap_err("Failed to get index")?;

        index
            .add_all(["."], git2::IndexAddOption::DEFAULT, None)
            .wrap_err("Failed to add all")?;
        index.write().wrap_err("Failed to write index")?;

        let oid = index.write_tree().wrap_err("Failed to write tree")?;
        let signature = self
            .repository
            .signature()
            .wrap_err("Failed to get signature")?;
        let tree = self
            .repository
            .find_tree(oid)
            .wrap_err("Failed to find tree")?;

        let message = "Falconf update";

        match self.repository.head() {
            Ok(_) => {
                debug!("Head exists");
                let parents = &[&self
                    .repository
                    .head()
                    .wrap_err("Failed to get head")?
                    .peel_to_commit()
                    .wrap_err("Failed to peel head to commit")?];

                self.repository
                    .commit(
                        Some("HEAD"),
                        &signature,
                        &signature,
                        message,
                        &tree,
                        parents,
                    )
                    .wrap_err("Failed to commit")?;
            }
            Err(_) => {
                debug!("Head doesn't exist, this is the initial commit");

                // Because there are no commits we need to make sure we're on `main` and not `master`
                self.repository
                    .set_head("refs/heads/main")
                    .wrap_err("Failed to set head to main")?;

                let parents = &[];
                self.repository.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    parents,
                )?;
            }
        }

        Ok(())
    }

    fn push(&self) -> Result<()> {
        // TODO: check for failed push
        //  > Note that you’ll likely want to use RemoteCallbacks and set push_update_reference
        //  > to test whether all the references were pushed successfully.
        //  And return PushPullError::divergence when it fails because of divergence in the remote
        let mut remote = self
            .repository
            .find_remote("origin")
            .wrap_err("Failed to find remote")?;
        remote
            .push(&[format!("refs/heads/{BRANCH}")], None)
            .wrap_err("Failed to push")?;
        Ok(())
    }

    pub fn pull_and_read(&mut self) -> Result<()> {
        self.pull().wrap_err("Failed to pull")?;
        self.update_data().wrap_err("Failed to update data")?;
        Ok(())
    }

    pub fn write_and_push(&self) -> Result<()> {
        self.write_data().wrap_err("Failed to write data")?;
        self.commit().wrap_err("Failed to commit")?;
        self.push().wrap_err("Failed to push")?;
        Ok(())
    }
}

fn data_path_from_repository(repo: &Repository) -> Result<PathBuf> {
    Ok(repo
        .workdir()
        .ok_or_eyre("Git repository is bare")?
        .join("data.json"))
}
