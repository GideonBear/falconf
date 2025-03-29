use crate::errors::ExecutionResult;
use crate::piece::Piece;
use crate::utils::press_enter;
use serde::{Deserialize, Serialize};

/// Request the user to perform an action manually *sad robot face*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manual {
    /// The message to show the user
    message: String,
}

impl Piece for Manual {
    fn execute(&self) -> ExecutionResult {
        Self::print_message(&self.message)
    }

    fn undo(&self) -> Option<ExecutionResult> {
        Some(Self::print_message(&format!(
            "UNDO the following change: {}",
            self.message
        )))
    }
}

impl Manual {
    fn print_message(message: &str) -> ExecutionResult {
        println!("Manual action required");
        println!("{}", message);
        println!("Continue when the action is performed.");
        press_enter()?;
        Ok(())
    }
}
