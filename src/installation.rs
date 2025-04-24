use crate::machine::Machine;
use crate::repo::Repo;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, WrapErr, eyre};
use std::env::home_dir;
use std::fs;
use std::path::PathBuf;

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

    fn get_root() -> Result<PathBuf> {
        Ok(home_dir().ok_or_eyre("No home dir found")?.join(".falconf"))
    }

    pub fn new(remote: &str) -> Result<Self> {
        let root = Self::get_root()?;
        if root.exists() {
            return Err(eyre!("Installation already exists"));
        }
        fs::create_dir(&root)?;

        let machine_path = root.join("machine");
        let repo_path = root.join("repo");

        let machine = Machine::new();
        fs::write(&machine_path, machine.0)?;

        let repo = Repo::new(remote, &repo_path)?;

        Ok(Self { machine, repo })
    }

    pub fn get() -> Result<Self> {
        let root = Self::get_root()?;

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
