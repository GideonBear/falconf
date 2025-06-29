use crate::installation::Installation;
use crate::machine::Machine;
use color_eyre::Result;
use std::path::PathBuf;

pub struct ExecutionData {
    pub file_dir: PathBuf,
    pub machine: Machine,
}

impl ExecutionData {
    pub fn new(installation: &Installation) -> Result<Self> {
        Ok(Self {
            file_dir: installation.repo().file_dir()?,
            machine: *installation.machine(),
        })
    }
}
