use crate::piece::{ExecutionResult, Piece, ResultExitStatusExt};
use crate::utils;

/// Run an arbitrary command with bash
#[derive(Debug, Clone)]
pub(crate) struct Command {
    /// The command to run
    command: String,
    /// If the command should be run with sudo
    sudo: bool,
    /// The command to run when undoing
    undo_command: Option<String>,
}

impl Piece for Command {
    fn execute(&self) -> ExecutionResult {
        Self::run_command(&self.command, self.sudo)
    }

    fn undo(&self) -> Option<ExecutionResult> {
        // This will return None if self.undo_command is None
        self.undo_command
            .as_ref()
            .map(|cmd| Self::run_command(cmd, self.sudo))
    }
}

impl Command {
    fn run_command(command: &str, sudo: bool) -> ExecutionResult {
        utils::if_sudo("bash", sudo)
            .arg("-c")
            .arg(command)
            .status()
            .to_execution_result()
    }
}
