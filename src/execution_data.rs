use crate::installation::Installation;
use color_eyre::Result;
use std::path::PathBuf;

pub struct ExecutionData {
    pub file_dir: PathBuf,
}

impl ExecutionData {
    pub fn new(installation: &Installation) -> Result<Self> {
        Ok(Self {
            file_dir: installation.repo().file_dir()?,
        })
    }
}
