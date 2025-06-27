use crate::cli::AddArgs;
use crate::execution_data::ExecutionData;
use crate::logging::CommandExt;
use crate::piece::Piece;
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

impl Piece for Apt {
    fn _execute(&self, _execution_data: &ExecutionData) -> Result<()> {
        // SAFETY: Since execute_bulk is implemented we assume this is never called.
        #[expect(clippy::panic)]
        {
            panic!();
        }
        // Self::execute_bulk(&[self])
    }

    fn execute_bulk(pieces: &[&Self], _execution_data: &ExecutionData) -> Result<()> {
        Self::apt_command(&["install"], pieces)
    }

    fn _undo(&self, _execution_data: &ExecutionData) -> Option<Result<()>> {
        // SAFETY: Since execute_bulk is implemented we assume this is never called.
        #[expect(clippy::panic)]
        {
            panic!();
        }
        // Some(Self::undo_bulk(&[self]))
    }

    fn undo_bulk(pieces: &[&Self], _execution_data: &ExecutionData) -> Result<()> {
        Self::apt_command(&["remove", "--autoremove"], pieces)
    }
}

impl Apt {
    fn apt_command(command: &[&str], pieces: &[&Self]) -> Result<()> {
        process::Command::new("apt")
            .args(command)
            .args(pieces.iter().map(|p| &p.package))
            .status_checked()?;
        Ok(())
    }

    pub fn from_cli(args: AddArgs) -> Result<Self> {
        if args.value.len() != 1 {
            return Err(eyre!(
                "Expected a singular value (package name) for 'apt' piece, got '{:?}'.",
                args.value
            ));
        }
        let package = args.value[0].clone();
        Ok(Self { package })
    }

    pub fn from_cli_autodetected(_args: AddArgs, package: String) -> Self {
        Self { package }
    }
}

impl Display for Apt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "apt install {}", self.package)
    }
}
