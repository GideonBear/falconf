use crate::cli::TopLevelArgs;
use crate::machine::Machine;
use crate::repo::Repo;
use color_eyre::Result;
use color_eyre::eyre::{WrapErr, eyre};
use log::debug;
use std::fs;

pub struct Installation {
    machine: Machine,
    repo: Repo,
}

impl Installation {
    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    pub fn repo(&mut self) -> &mut Repo {
        &mut self.repo
    }

    pub fn new(remote: &str, top_level_args: &TopLevelArgs) -> Result<Self> {
        let root = &top_level_args.path;
        debug!("Looking at {root:?}");

        if root.try_exists()? {
            return Err(eyre!("Installation already exists"));
        }
        fs::create_dir(root)?;

        let machine_path = root.join("machine");
        let repo_path = root.join("repo");

        let machine = Machine::new();
        fs::write(&machine_path, machine.0)?;

        let repo = Repo::new(remote, &repo_path)?;

        Ok(Self { machine, repo })
    }

    pub fn get(top_level_args: &TopLevelArgs) -> Result<Self> {
        let root = &top_level_args.path;
        debug!("Looking at {root:?}");

        if !root.is_dir() {
            return Err(eyre!("No installation found"));
        }

        let machine = Machine(
            fs::read_to_string(root.join("machine"))?
                .parse()
                .wrap_err("`machine` file does not contain a valid UUID".to_string())?,
        );

        let repo = Repo::from_path(&root.join("repo"))?;

        Ok(Self { machine, repo })
    }
}
