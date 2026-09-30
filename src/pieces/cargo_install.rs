use crate::cli::add;
use crate::execution_data::ExecutionData;
use crate::logging::CommandExt;
use crate::piece::BulkPiece;
use crate::utils::which;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoInstall {
    /// The crate to install
    crate_: String,
}

impl BulkPiece for CargoInstall {
    fn execute_bulk(pieces: &[&mut Self], _execution_data: &ExecutionData) -> Result<()> {
        let mut cmd = Command::new("cargo");
        if which("cargo-binstall")?.is_some() {
            cmd.arg("binstall");
        } else {
            cmd.arg("install");
        }
        cmd.arg("-y");
        cmd.args(pieces.iter().map(|p| &p.crate_));
        cmd.status_checked()?;
        Ok(())
    }

    fn undo_bulk(pieces: &[&mut Self], _execution_data: &ExecutionData) -> Result<()> {
        Command::new("cargo")
            .arg("uninstall")
            .args(pieces.iter().map(|p| &p.crate_))
            .status_checked()?;
        Ok(())
    }
}

impl CargoInstall {
    pub fn from_cli(args: &add::Args) -> Result<Self> {
        if args.value.len() != 1 {
            return Err(eyre!(
                "Expected a singular value (crate name) for 'cargo-install' piece, got '{:?}'.",
                args.value
            ));
        }
        let crate_ = args.value[0].clone();
        Ok(Self { crate_ })
    }

    pub const fn from_cli_autodetected(_args: &add::Args, crate_: String) -> Self {
        Self { crate_ }
    }
}

impl Display for CargoInstall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "cargo install {}", self.crate_)
    }
}
