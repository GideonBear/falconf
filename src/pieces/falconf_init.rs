use crate::execution_context::ExecutionContext;
use crate::piece::{NonBulkPiece, Piece};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use shell_words::quote;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct FalconfInit {
    remote: String,
}

impl Piece for FalconfInit {
    fn seed(&self, _ctx: &ExecutionContext) -> Result<(String, bool, Option<String>)> {
        Ok((format!("falconf init {}", quote(&self.remote)), true, None))
    }
}

impl NonBulkPiece for FalconfInit {
    fn execute(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Err(eyre!(
            "It doesn't make any sense to execute the falconf init piece"
        ))
    }

    fn undo(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Err(eyre!(
            "It doesn't make any sense to undo the falconf init piece"
        ))
    }
}

impl FalconfInit {
    pub fn new(remote: String) -> Self {
        Self { remote }
    }
}

impl Display for FalconfInit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "falconf init {}", self.remote)
    }
}
