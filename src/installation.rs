use crate::cli::TopLevelArgs;
use crate::machine::{Machine, MachineData};
use crate::repo::Repo;
use color_eyre::Result;
use color_eyre::eyre::{WrapErr, eyre};
use log::{debug, info};
use std::fs;
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};

pub struct Installation {
    machine: Machine,
    repo: Repo,
}

impl Installation {
    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    pub fn repo(&self) -> &Repo {
        &self.repo
    }

    pub fn repo_mut(&mut self) -> &mut Repo {
        &mut self.repo
    }

    pub fn init(top_level_args: &TopLevelArgs, remote: &str, new: bool) -> Result<()> {
        match Self::_init(top_level_args, remote, new) {
            Ok(()) => Ok(()),
            Err(e) => {
                info!(
                    "Found error during init; removing newly created .falconf directory to avoid half-initialized state"
                );
                remove_dir_all(top_level_args.path.clone())?;
                Err(e)
            }
        }
    }

    fn _init(top_level_args: &TopLevelArgs, remote: &str, new: bool) -> Result<()> {
        let root = &top_level_args.path;
        debug!("Looking at {root:?}");

        if root.try_exists()? {
            return Err(eyre!("Installation already exists"));
        }
        fs::create_dir(root)?;

        let machine_path = root.join("machine");
        let repository_path = Self::get_repository_path(root);

        let machine = Machine::new();
        fs::write(&machine_path, machine.0.to_string())?;
        let machine_data = MachineData::new_this()?;

        Repo::init(remote, &repository_path, machine, machine_data, new)?;

        Ok(())
    }

    pub fn get(top_level_args: &TopLevelArgs) -> Result<Self> {
        let root = &top_level_args.path;
        debug!("Looking at {root:?}");

        if !root.is_dir() {
            return Err(eyre!(
                "No installation found at {root:?}. Run `falconf init` first!"
            ));
        }

        let machine = Machine(
            fs::read_to_string(root.join("machine"))?
                .parse()
                .wrap_err("`machine` file does not contain a valid UUID".to_string())?,
        );

        let repo = Repo::get_from_path(&Self::get_repository_path(root))?;

        Ok(Self { machine, repo })
    }

    fn get_repository_path(root: &Path) -> PathBuf {
        root.join("repository")
    }
}
