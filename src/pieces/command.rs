use crate::cli::add;
use crate::execution_context::ExecutionContext;
use crate::logging::CommandExt as _;
use crate::piece::{NonBulkPiece, Piece};
use crate::utils::{join_value, prompt};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::process;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Command {
    /// The command to run
    command: String,
    // TODO(test): test the undo_command
    /// The command to run when undoing
    pub undo_command: Option<String>,
}

impl Piece for Command {
    fn seed(&self, _ctx: &ExecutionContext) -> Result<(String, bool, Option<String>)> {
        Ok((self.command.clone(), true, None))
    }
}

impl NonBulkPiece for Command {
    fn execute(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Self::run_command(&self.command)
    }

    fn undo(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        if self.undo_command.is_none() {
            let undo_command =
                prompt("This command piece is missing an undo command. Undo command to use: ")?;
            self.undo_command = Some(undo_command);
        }
        // TODO(low): do this in a non-unwrappy way
        Self::run_command(self.undo_command.as_ref().unwrap())
    }
}

impl Command {
    fn run_command(command: &str) -> Result<()> {
        process::Command::new("bash")
            .arg("-c")
            .arg(command)
            .status_checked()?;
        Ok(())
    }

    pub fn from_cli(args: &add::Args) -> Self {
        Self {
            command: join_value(&args.value),
            undo_command: args.undo.clone(),
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.command)
    }
}
