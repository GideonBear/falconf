use crate::cli::AddArgs;
use crate::execution_data::ExecutionData;
use crate::logging::CommandExt;
use crate::piece::Piece;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::process;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// The command to run
    command: String,
    /// The command to run when undoing
    undo_command: Option<String>,
}

impl Piece for Command {
    fn _execute(&self, _execution_data: &ExecutionData) -> Result<()> {
        Self::run_command(&self.command)
    }

    fn _undo(&self, _execution_data: &ExecutionData) -> Option<Result<()>> {
        // This will return None if self.undo_command is None
        self.undo_command.as_ref().map(|cmd| Self::run_command(cmd))
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

    pub fn from_cli(args: AddArgs) -> Self {
        Self {
            // TODO: Extensive testing of this
            command: shell_words::join(args.value),
            undo_command: None,
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.command)
    }
}
