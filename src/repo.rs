use crate::data::Data;
use crate::machine::{Machine, MachineData, machines_by_name};
use crate::utils::remove_empty_dirs;
use crate::{data, migrations};
use auth_git2::GitAuthenticator;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt as _, WrapErr as _, eyre};
use git2::build::CheckoutBuilder;
use git2::{
    CheckoutNotificationType, Commit, Diff, DiffOptions, Error, Repository, ResetType, Status,
    StatusOptions, Tree,
};
use itertools::Itertools as _;
use log::debug;
use std::fmt::{Debug, Formatter};
use std::fs::{File, create_dir};
use std::path::{Path, PathBuf};

const BRANCH: &str = "main";
const BRANCH_REF: &str = "refs/heads/main";

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
    ) -> Result<Self> {
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
        if machines_by_name(data.machines()).contains_key(machine_data.name.as_str()) {
            // TODO: allow passing machine name with `--name` to `falconf init`
            return Err(eyre!(
                "Machine name '{}' is already taken",
                machine_data.name
            ));
        }
        data.machines_mut().insert(machine, machine_data);
        repo.write_and_push(files, None)
            .wrap_err("Failed to write_and_push")?;
        Ok(repo)
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

    fn from_repository(repository: Repository) -> Result<Self> {
        let auth = GitAuthenticator::default();
        let needs_push = Self::pull(&repository, &auth).wrap_err("Failed to pull")?;
        let data_path = data_path_from_repository(&repository)?;
        let migrated = migrations::run_migrations(&data_path)?;
        let data = data::from_file::<Data>(&data_path).wrap_err("Failed to get data")?;

        let repo = Self {
            repository,
            auth,
            data,
        };
        if needs_push {
            debug!("Local branch is ahead of the remote; pushing");
            push(&repo.repository, &repo.auth).wrap_err("Failed to push")?;
        }
        if let Some(migrated) = migrated {
            repo.write_and_push(vec![], Some(migrated))?
        }
        // This runs at the start of every run, so we do sanity checks here
        if repo.data_changed()? {
            return Err(eyre!("The data file has uncommitted changes"));
        }

        Ok(repo)
    }

    /// Commit all local changes in the file dir, then pull (merging if necessary) and push.
    /// If anything before the push fails, the local commit is undone, so the local changes are
    /// left exactly as they were. `confirm` is called with the diff of the local changes, and
    /// can abort by returning false. Returns false if there were no changes.
    pub fn push_local_changes(
        path: &Path,
        confirm: impl FnOnce(&Diff<'_>) -> Result<bool>,
    ) -> Result<bool> {
        let repository = Repository::open(path).wrap_err("Failed to open repository")?;
        let auth = GitAuthenticator::default();

        if data_changed(&repository)? {
            return Err(eyre!("The data file has uncommitted changes"));
        }

        let diff = diff_index_to_workdir(&repository)?;
        let files: Vec<String> = diff
            .deltas()
            .filter_map(|d| d.new_file().path())
            .map(|path| {
                if path.starts_with("files") {
                    Ok(path.to_string_lossy().to_string())
                } else {
                    Err(eyre!("A file not in files/ was changed in the repo"))
                }
            })
            .collect::<Result<_>>()?;

        if files.is_empty() {
            return Ok(false);
        }
        if !confirm(&diff)? {
            return Err(eyre!("Aborted"));
        }

        let old_head = repository
            .head()
            .wrap_err("Failed to get head")?
            .peel_to_commit()
            .wrap_err("Failed to peel head to commit")?;
        commit(&repository, files, None).wrap_err("Failed to commit")?;
        if let Err(err) = Self::pull(&repository, &auth) {
            debug!("Pull failed, undoing the local commit");
            repository
                .reset(old_head.as_object(), ResetType::Mixed, None)
                .wrap_err("Failed to undo the local commit after failing to pull")?;
            return Err(err.wrap_err("Failed to pull"));
        }
        push(&repository, &auth).wrap_err("Failed to push")?;

        Ok(true)
    }

    /// Fetch and integrate the remote branch, never overwriting local changes.
    /// Returns true if the local branch is ahead of the remote, and needs to be pushed.
    fn pull(repository: &Repository, auth: &GitAuthenticator) -> Result<bool> {
        let mut remote = repository
            .find_remote("origin")
            .wrap_err("Failed to find remote")?;
        auth.fetch(repository, &mut remote, &[BRANCH], None)
            .wrap_err("Failed to fetch")?;

        let fetch_head = repository
            .find_reference("FETCH_HEAD")
            .wrap_err("Failed to find fetch head")?;
        let fetch_commit = repository
            .reference_to_annotated_commit(&fetch_head)
            .wrap_err("Failed to convert fetch head to annotated commit")?;
        let theirs = fetch_head
            .peel_to_commit()
            .wrap_err("Failed to peel fetch head to commit")?;
        let (analysis, _preference) = repository
            .merge_analysis(&[&fetch_commit])
            .wrap_err("Failed to do merge analysis")?;

        if analysis.is_up_to_date() {
        } else if analysis.is_fast_forward() {
            debug!("Fast-forwarding");
            checkout_safe(repository, &theirs.tree()?)?;
            let mut reference = repository.find_reference(BRANCH_REF)?;
            reference.set_target(theirs.id(), "falconf: fast-forward")?;
            repository.set_head(BRANCH_REF)?;
        } else {
            debug!("Branches have diverged; merging");
            merge(repository, &theirs)?;
        }

        let head = repository
            .head()
            .wrap_err("Failed to get head")?
            .peel_to_commit()
            .wrap_err("Failed to peel head to commit")?;
        Ok(head.id() != theirs.id())
    }

    fn write_data(&self) -> Result<()> {
        data::to_file(&self.data, &data_path_from_repository(&self.repository)?)
    }

    /// Returns true if the data file was changed
    fn data_changed(&self) -> Result<bool> {
        data_changed(&self.repository)
    }

    /// `files`: A list of files relative to the file dir that will be committed along with the data file.
    fn commit(&self, files: Vec<PathBuf>, extra_msg: Option<String>) -> Result<()> {
        let file_dir = self.file_dir()?;
        #[expect(clippy::missing_panics_doc, reason = "see expect")]
        let file_dir = file_dir
            .strip_prefix(self.workdir()?)
            .expect("File dir is always within repository workdir");
        let files = files
            .into_iter()
            .map(|p| file_dir.join(p).to_string_lossy().to_string())
            .collect();
        commit(&self.repository, files, extra_msg)
    }

    fn push(&self) -> Result<()> {
        push(&self.repository, &self.auth)
    }

    pub fn write_and_push(&self, files: Vec<PathBuf>, extra_msg: Option<String>) -> Result<()> {
        // If the data file changed or there are other files to commit
        self.write_data().wrap_err("Failed to write data")?;
        if self.data_changed()? || !files.is_empty() {
            // if args.dry_run {
            //     return Err(eyre!(
            //         "Somehow, the data file was changed during a dry run. This shouldn't happen."
            //     ));
            // }
            self.commit(files, extra_msg).wrap_err("Failed to commit")?;
            self.push().wrap_err("Failed to push")?;
        }
        Ok(())
    }

    /// Returns true if `file` (relative to the file dir) has uncommitted changes
    pub fn file_has_changes(&self, file: &Path) -> Result<bool> {
        let file_dir = self.file_dir()?;
        #[expect(clippy::missing_panics_doc, reason = "see expect")]
        let path = file_dir
            .strip_prefix(self.workdir()?)
            .expect("File dir is always within repository workdir")
            .join(file);
        let statuses = self
            .repository
            .statuses(Some(
                StatusOptions::new()
                    .pathspec(&path)
                    .disable_pathspec_match(true)
                    .include_untracked(true)
                    .recurse_untracked_dirs(true)
                    .include_ignored(false),
            ))
            .wrap_err("Failed to get status")?;
        Ok(!statuses.is_empty())
    }

    pub fn clean_file_dir(&self) -> Result<()> {
        remove_empty_dirs(&self.file_dir()?)
    }
}

/// Check out `tree`, failing without changing anything if that would overwrite local changes.
fn checkout_safe(repository: &Repository, tree: &Tree<'_>) -> Result<()> {
    let mut conflicts = vec![];
    let result = {
        let mut builder = CheckoutBuilder::new();
        builder
            .safe()
            .overwrite_ignored(false)
            .notify_on(CheckoutNotificationType::CONFLICT)
            .notify(|_kind, path, _baseline, _target, _workdir| {
                if let Some(path) = path {
                    conflicts.push(path.display().to_string());
                }
                true
            });
        repository.checkout_tree(tree.as_object(), Some(&mut builder))
    };
    if !conflicts.is_empty() {
        return Err(eyre!(
            "Pulling would overwrite local changes to: {}. Nothing was changed. Run `falconf push` to push your changes first, or revert them.",
            conflicts.iter().join(", ")
        ));
    }
    result.wrap_err("Failed to checkout")
}

/// Merge `theirs` into HEAD and commit, failing without changing anything on conflicts.
fn merge(repository: &Repository, theirs: &Commit<'_>) -> Result<()> {
    let ours = repository
        .head()
        .wrap_err("Failed to get head")?
        .peel_to_commit()
        .wrap_err("Failed to peel head to commit")?;
    let mut index = repository
        .merge_commits(&ours, theirs, None)
        .wrap_err("Failed to merge")?;
    if index.has_conflicts() {
        let conflicts = index
            .conflicts()?
            .map(|conflict| {
                let conflict = conflict?;
                let entry = conflict
                    .our
                    .or(conflict.their)
                    .or(conflict.ancestor)
                    .ok_or_eyre("Conflict without entries")?;
                Ok(String::from_utf8_lossy(&entry.path).to_string())
            })
            .collect::<Result<Vec<_>>>()?;
        return Err(eyre!(
            "Local changes conflict with remote changes in: {}. Nothing was changed. Resolve this manually with git in {}.",
            conflicts.iter().join(", "),
            workdir_from_repository(repository)?.display()
        ));
    }
    let tree = repository
        .find_tree(index.write_tree_to(repository)?)
        .wrap_err("Failed to find merged tree")?;
    checkout_safe(repository, &tree)?;
    let signature = repository.signature().wrap_err("Failed to get signature")?;
    repository
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            "falconf: Merge remote changes",
            &tree,
            &[&ours, theirs],
        )
        .wrap_err("Failed to commit merge")?;
    Ok(())
}

fn data_changed(repository: &Repository) -> Result<bool> {
    Ok(repository
        .status_file(DATA_PATH.as_ref())
        .wrap_err("Failed to get status from data file")?
        .contains(Status::WT_MODIFIED))
}

fn diff_index_to_workdir(repository: &Repository) -> std::result::Result<Diff<'_>, Error> {
    repository.diff_index_to_workdir(
        None,
        Some(
            DiffOptions::new()
                .include_untracked(true)
                .recurse_untracked_dirs(true)
                .show_untracked_content(true),
        ),
    )
}

/// `files`: A list of files relative to the workdir that will be committed along with the data file.
fn commit(
    repository: &Repository,
    mut files: Vec<String>,
    extra_msg: Option<String>,
) -> Result<()> {
    let mut index = repository.index().wrap_err("Failed to get index")?;

    files.push(DATA_PATH.to_owned());
    index
        .add_all(&files, git2::IndexAddOption::DEFAULT, None)
        .wrap_err("Failed to add all")?;
    index.write().wrap_err("Failed to write index")?;

    let oid = index.write_tree().wrap_err("Failed to write tree")?;
    let signature = repository.signature().wrap_err("Failed to get signature")?;
    let tree = repository.find_tree(oid).wrap_err("Failed to find tree")?;

    let mut message = format!("falconf: Update {}", files.iter().join(", "));
    if let Some(extra_msg) = extra_msg {
        message.push_str(&format!(" ({extra_msg})"));
    }

    if repository.head().is_ok() {
        debug!("Head exists");
        let parents = &[&repository
            .head()
            .wrap_err("Failed to get head")?
            .peel_to_commit()
            .wrap_err("Failed to peel head to commit")?];

        repository
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
        repository
            .set_head(BRANCH_REF)
            .wrap_err("Failed to set head to main")?;

        let parents = &[];
        repository.commit(
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

fn push(repository: &Repository, auth: &GitAuthenticator) -> Result<()> {
    // TODO(high):
    //  check for failed push
    //  > Note that you’ll likely want to use RemoteCallbacks and set push_update_reference
    //  > to test whether all the references were pushed successfully.
    //  And return a divergence error when it fails because of divergence in the remote
    let mut remote = repository
        .find_remote("origin")
        .wrap_err("Failed to find remote")?;
    auth.push(repository, &mut remote, &[BRANCH_REF])
        .wrap_err("Failed to push")?;
    Ok(())
}

// TODO(low): below three are a bit convoluted

pub const DATA_PATH: &str = "data.ron";

fn workdir_from_repository(repo: &Repository) -> Result<&Path> {
    repo.workdir().ok_or_eyre("Repository is bare")
}

fn data_path_from_repository(repo: &Repository) -> Result<PathBuf> {
    Ok(workdir_from_repository(repo)?.join(DATA_PATH))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::TopLevelArgs;
    use crate::installation::Installation;
    use crate::testing::TestEnv;
    use std::fs::OpenOptions;
    use std::io::Write;

    #[test]
    fn test_error_on_changed_data_file() -> Result<()> {
        let mut env = TestEnv::new(1)?;

        env.with(0, |f| {
            // Write a single newline to the end
            let mut file = OpenOptions::new()
                .append(true)
                .open(f.join("repository").join(DATA_PATH))?;
            writeln!(file)?;
            // It should now crash
            let top_level_args = TopLevelArgs::new_testing(f.clone(), true);
            assert_eq!(
                Installation::get(&top_level_args).unwrap_err().to_string(),
                "The data file has uncommitted changes"
            );
            Ok(())
        })
    }
}
