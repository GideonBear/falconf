use crate::cli::add;
use crate::execution_data::ExecutionData;
use crate::logging::CommandExt as _;
use crate::piece::BulkPiece;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::process;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Apt {
    /// The package to install
    package: String,
}

impl BulkPiece for Apt {
    fn execute_bulk(pieces: &[&mut Self], _execution_data: &ExecutionData) -> Result<()> {
        Self::apt_command(&["install"], pieces)
    }

    fn undo_bulk(pieces: &[&mut Self], _execution_data: &ExecutionData) -> Result<()> {
        Self::apt_command(&["remove", "--autoremove"], pieces)
    }
}

impl Apt {
    fn apt_command(command: &[&str], pieces: &[&mut Self]) -> Result<()> {
        process::Command::new("apt")
            .args(command)
            .args(pieces.iter().map(|p| &p.package))
            .status_checked()?;
        Ok(())
    }

    pub fn from_cli(args: &add::Args) -> Result<Self> {
        if args.value.len() != 1 {
            return Err(eyre!(
                "Expected a singular value (package name) for 'apt' piece, got '{:?}'.",
                args.value
            ));
        }
        let package = args.value[0].clone();
        Ok(Self { package })
    }

    pub const fn from_cli_autodetected(_args: &add::Args, package: String) -> Self {
        Self { package }
    }
}

impl Display for Apt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "apt install {}", self.package)
    }
}
