use crate::cli::AddArgs;
use crate::logging::CommandExt;
use crate::piece::Piece;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::process;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Apt {
    /// The package to install
    package: String,
}

impl Piece for Apt {
    fn _execute(&self) -> Result<()> {
        // Since execute_bulk is implemented we assume this is never called.
        panic!();
        // Self::execute_bulk(&[self])
    }

    fn execute_bulk(pieces: &[&Self]) -> Result<()> {
        Self::apt_command(&["install"], pieces)
    }

    fn _undo(&self) -> Option<Result<()>> {
        // Since execute_bulk is implemented we assume this is never called.
        panic!();
        // Some(Self::undo_bulk(&[self]))
    }

    fn undo_bulk(pieces: &[&Self]) -> Result<()> {
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
