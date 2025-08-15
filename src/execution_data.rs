use crate::cli::TopLevelArgs;
use crate::installation::Installation;
use crate::machine::Machine;
use color_eyre::Result;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ExecutionData {
    pub file_dir: PathBuf,
    pub machine: Machine,
    pub dry_run: bool,
    pub test_run: bool,
}

impl ExecutionData {
    pub fn new(installation: &Installation, top_level_args: &TopLevelArgs) -> Result<Self> {
        Ok(Self {
            file_dir: installation.repo().file_dir()?,
            machine: *installation.machine(),
            dry_run: top_level_args.dry_run,
            test_run: top_level_args.test_run,
        })
    }
}
