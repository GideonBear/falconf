use crate::cli::TopLevelArgs;
use crate::installation::Installation;
use crate::machine::{Machine, MachineData};
use color_eyre::Result;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ExecutionContext {
    pub file_dir: PathBuf,
    pub machine: Machine,
    pub machine_data: MachineData,
    // pub dry_run: bool,
    pub test_run: bool,
}

impl ExecutionContext {
    pub fn new(installation: &Installation, top_level_args: &TopLevelArgs) -> Result<Self> {
        let machine = installation.machine();
        Ok(Self {
            file_dir: installation.repo().file_dir()?,
            machine,
            #[expect(clippy::missing_panics_doc, reason = "Invalid config")]
            machine_data: installation
                .repo()
                .data()
                .machines()
                .get(&machine)
                .unwrap()
                .clone(),
            // dry_run: top_level_args.dry_run,
            test_run: top_level_args.test_run,
        })
    }
}
