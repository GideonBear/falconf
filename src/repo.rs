use crate::data::Data;
use crate::machine::{Machine, MachineData};
use crate::utils::remove_empty_dirs;
use auth_git2::GitAuthenticator;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt as _, WrapErr as _, eyre};
use git2::{Diff, Error, Repository, Status};
use itertools::Itertools as _;
use log::debug;
use std::fmt::{Debug, Formatter};
use std::fs::{File, create_dir};
use std::path::{Path, PathBuf};

const BRANCH: &str = "main";

pub struct Repo {
    repository: Repository,
    auth: GitAuthenticator,
    data: Data,
}

impl Debug for Repo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Repo {{ path: {:?}, data: {:?} }}",
            self.workdir().ok(),
            self.data,
        )
    }
}

// TODO(low): instead of write_and_push, make function that takes in a closure gets a &mut Data?
//  After the closure, write it. This makes sure that we never forget to write.
//  see iso-updater
//  This should also remove duplication of repo.pull_and_read and repo.write_and_push from cli functions

impl Repo {
    pub fn init(
        remote: &str,
        path: &Path,
        machine: Machine,
        machine_data: MachineData,
        new: bool,
    ) -> Result<()> {
        let auth = GitAuthenticator::default();
        debug!("Cloning repo");
        let repository = auth
            .clone_repo(remote, path)
            .wrap_err("Failed to clone repository")?;

        let mut files = vec![];

        let mut repo = if new {
            debug!("New, so initializing new");
            let data = Data::init_new();
            let repo = Self {
                repository,
                auth,
                data,
            };

            let file_dir = repo.file_dir().wrap_err("Failed to get file dir")?;
            create_dir(&file_dir).wrap_err("Failed to create file dir")?;
            let gitkeep = file_dir.join(".gitkeep");
            files.push(".gitkeep".into());
            File::create(gitkeep).wrap_err("Failed to create .gitkeep")?;

            // We push below
            repo
        } else {
            debug!("Not new, so initializing existing");
            let data_path = data_path_from_repository(&repository)?;
            if !data_path.exists() {
                return Err(eyre!(
                    "This is not a falconf repo. Maybe you forgot `--new`? ({data_path:?} does not exist)"
                ));
            }
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
        repo.write_and_push(files)
            .wrap_err("Failed to write_and_push")?;
        Ok(())
    }

    pub fn workdir(&self) -> Result<&Path> {
        workdir_from_repository(&self.repository)
    }

    pub fn file_dir(&self) -> Result<PathBuf> {
        Ok(self.workdir()?.join("files"))
    }

    pub const fn data(&self) -> &Data {
        &self.data
    }

    pub const fn data_mut(&mut self) -> &mut Data {
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
        let auth = GitAuthenticator::default();
        let data = Self::get_data(&repository).wrap_err("Failed to get data")?;

        let repo = Self {
            repository,
            auth,
            data,
        };
        // This runs at the start of every run, so we do sanity checks here
        if repo.data_changed()? {
            return Err(eyre!("The data file has uncommitted changes"));
        }

        Ok(repo)
    }

    fn pull(&self) -> Result<()> {
        let mut remote = self
            .repository
            .find_remote("origin")
            .wrap_err("Failed to find remote")?;
        self.auth
            .fetch(&self.repository, &mut remote, &[BRANCH], None)
            .wrap_err("Failed to fetch")?;

        let fetch_head = self
            .repository
            .find_reference("FETCH_HEAD")
            .wrap_err("Failed to find fetch head")?;
        let fetch_commit = self
            .repository
            .reference_to_annotated_commit(&fetch_head)
            .wrap_err("Failed to convert fetch head to annotated commit")?;
        let (analysis, _preference) = self
            .repository
            .merge_analysis(&[&fetch_commit])
            .wrap_err("Failed to do merge analysis")?;

        if analysis.is_up_to_date() {
            Ok(())
        } else if analysis.is_fast_forward() {
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
            .to_file(&data_path_from_repository(&self.repository)?)
    }

    /// Returns true if the data file was changed
    fn data_changed(&self) -> Result<bool> {
        Ok(self
            .repository
            .status_file(DATA_PATH.as_ref())
            .wrap_err("Failed to get status from data file")?
            .contains(Status::WT_MODIFIED))
    }

    /// `files`: A list of files relative to the file dir that will be committed along with the data file.
    fn commit(&self, files: Vec<PathBuf>) -> Result<()> {
        let mut index = self.repository.index().wrap_err("Failed to get index")?;

        let file_dir = self.file_dir()?;
        #[expect(clippy::missing_panics_doc, reason = "see expect")]
        let file_dir = file_dir
            .strip_prefix(self.workdir()?)
            .expect("File dir is always within repository workdir");
        let mut files = files
            .into_iter()
            .map(|p| file_dir.join(p).to_string_lossy().to_string())
            .collect::<Vec<_>>();
        files.push(DATA_PATH.to_owned());
        index
            .add_all(&files, git2::IndexAddOption::DEFAULT, None)
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

        let message = format!("falconf: Update {}", files.iter().join(", "));

        if self.repository.head().is_ok() {
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
                    &message,
                    &tree,
                    parents,
                )
                .wrap_err("Failed to commit")?;
        } else {
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
                &message,
                &tree,
                parents,
            )?;
        }

        Ok(())
    }

    fn push(&self) -> Result<()> {
        // TODO(high):
        //  I need to test what happens right now if you forget to `falconf push` and do a pull.
        //  check for failed push
        //  > Note that you’ll likely want to use RemoteCallbacks and set push_update_reference
        //  > to test whether all the references were pushed successfully.
        //  And return a divergence error when it fails because of divergence in the remote
        let mut remote = self
            .repository
            .find_remote("origin")
            .wrap_err("Failed to find remote")?;
        self.auth
            .push(
                &self.repository,
                &mut remote,
                &[&format!("refs/heads/{BRANCH}")],
            )
            .wrap_err("Failed to push")?;
        Ok(())
    }

    pub fn pull_and_read(&mut self) -> Result<()> {
        self.pull().wrap_err("Failed to pull")?;
        self.update_data().wrap_err("Failed to update data")?;
        Ok(())
    }

    pub fn write_and_push(&self, files: Vec<PathBuf>) -> Result<()> {
        // If the data file changed or there are other files to commit
        self.write_data().wrap_err("Failed to write data")?;
        if self.data_changed()? || !files.is_empty() {
            // if args.dry_run {
            //     return Err(eyre!(
            //         "Somehow, the data file was changed during a dry run. This shouldn't happen."
            //     ));
            // }
            self.commit(files).wrap_err("Failed to commit")?;
            self.push().wrap_err("Failed to push")?;
        }
        Ok(())
    }

    pub fn diff_index_to_workdir(&self) -> std::result::Result<Diff<'_>, Error> {
        self.repository.diff_index_to_workdir(None, None)
    }

    pub fn clean_file_dir(&self) -> Result<()> {
        remove_empty_dirs(&self.file_dir()?)
    }
}

// TODO(low): below three are a bit convoluted

const DATA_PATH: &str = "data.ron";

fn workdir_from_repository(repo: &Repository) -> Result<&Path> {
    repo.workdir().ok_or_eyre("Repository is bare")
}

fn data_path_from_repository(repo: &Repository) -> Result<PathBuf> {
    Ok(workdir_from_repository(repo)?.join(DATA_PATH))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::cli::TopLevelArgs;
    use crate::cli::init::tests::init_util;
    use crate::installation::Installation;
    use crate::testing::TestRemote;
    use std::fs::OpenOptions;
    use std::io::Write;

    #[test]
    fn test_error_on_changed_data_file() -> Result<()> {
        let remote = TestRemote::new()?;
        let local = init_util(&remote, true)?;
        // Write a single newline to the end
        let mut file = OpenOptions::new()
            .append(true)
            .open(local.path().join("repository").join(DATA_PATH))?;
        writeln!(file)?;
        // It should now crash
        let top_level_args = TopLevelArgs::new_testing(local.path().clone(), true);
        assert_eq!(
            Installation::get(&top_level_args).unwrap_err().to_string(),
            "The data file has uncommitted changes"
        );

        Ok(())
    }
}
