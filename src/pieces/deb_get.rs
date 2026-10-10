use crate::cli::add;
use crate::execution_context::ExecutionContext;
use crate::logging::CommandExt as _;
use crate::piece::{BulkPiece, Piece};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use shell_words::quote;
use std::fmt::{Display, Formatter};
use std::process;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DebGet {
    /// The package to install
    package: String,
}

impl Piece for DebGet {
    fn seed(&self, _ctx: &ExecutionContext) -> Result<(String, bool, Option<String>)> {
        Ok((
            format!("deb-get install {}", quote(&self.package)),
            true,
            None,
        ))
    }
}

impl BulkPiece for DebGet {
    fn execute_bulk(pieces: &[&mut Self], _ctx: &ExecutionContext) -> Result<()> {
        Self::deb_get_command(&["install"], pieces)
    }

    fn undo_bulk(pieces: &[&mut Self], _ctx: &ExecutionContext) -> Result<()> {
        Self::deb_get_command(&["remove", "--remove-repo"], pieces)
    }
}

impl DebGet {
    fn deb_get_command(command: &[&str], pieces: &[&mut Self]) -> Result<()> {
        process::Command::new("deb-get")
            .args(command)
            .args(pieces.iter().map(|p| &p.package))
            .status_checked()?;
        Ok(())
    }

    pub fn from_cli(args: &add::Args) -> Result<Self> {
        if args.value.len() != 1 {
            return Err(eyre!(
                "Expected a singular value (package name) for 'deb-get' piece, got '{:?}'.",
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

impl Display for DebGet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "deb-get install {}", self.package)
    }
}
