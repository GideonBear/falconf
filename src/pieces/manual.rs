use crate::cli::AddArgs;
use crate::execution_data::ExecutionData;
use crate::piece::Piece;
use crate::utils::press_enter;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manual {
    /// The message to show the user
    message: String,
}

impl Piece for Manual {
    fn _execute(&self, _execution_data: &ExecutionData) -> Result<()> {
        Self::print_message(&self.message)
    }

    fn _undo(&self, _execution_data: &ExecutionData) -> Option<Result<()>> {
        Some(Self::print_message(&format!(
            "UNDO the following change: {}",
            self.message
        )))
    }
}

impl Manual {
    #[allow(clippy::print_stdout)] // TODO
    fn print_message(message: &str) -> Result<()> {
        println!("Manual action required");
        println!("{message}");
        println!("Continue when the action is performed.");
        press_enter()?;
        Ok(())
    }

    pub fn from_cli(args: &AddArgs) -> Self {
        Self {
            message: args.value.join(" "),
        }
    }
}

impl Display for Manual {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Manual action: {}", self.message)
    }
}
