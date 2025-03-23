use crate::piece::{ExecutionResult, Piece};
use crate::utils::press_enter;

/// Request the user to perform an action manually *sad robot face*
pub(crate) struct Manual {
    /// The message to show the user
    message: String,
}

impl Piece for Manual {
    fn execute(&self) -> ExecutionResult {
        Self::print_message(&self.message);
        Ok(())
    }

    fn undo(&self) -> Option<ExecutionResult> {
        Self::print_message(&format!("UNDO the following change: {}", self.message));
        Some(Ok(()))
    }
}

impl Manual {
    fn print_message(message: &str) {
        println!("Manual action required");
        println!("{}", message);
        println!("Continue when the action is performed.");
        press_enter();
    }
}