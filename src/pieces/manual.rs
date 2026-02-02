use crate::cli::add;
use crate::execution_data::ExecutionData;
use crate::piece::NonBulkPiece;
use crate::utils::press_enter;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manual {
    /// The message to show the user
    message: String,
}

impl NonBulkPiece for Manual {
    fn execute(&mut self, _execution_data: &ExecutionData) -> Result<()> {
        Self::print_message(&self.message)
    }

    fn undo(&mut self, _execution_data: &ExecutionData) -> Result<()> {
        Self::print_message(&format!("UNDO the following change: {}", self.message))
    }
}

impl Manual {
    #[expect(clippy::print_stdout)]
    fn print_message(message: &str) -> Result<()> {
        println!("Manual action required");
        println!("{message}");
        println!("Continue when the action is performed.");
        press_enter()?;
        Ok(())
    }

    pub fn from_cli(args: &add::Args) -> Self {
        Self {
            message: shell_words::join(&args.value),
        }
    }
}

impl Display for Manual {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Manual action: {}", self.message)
    }
}
