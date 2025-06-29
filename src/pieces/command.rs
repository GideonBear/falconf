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

    pub fn from_cli(args: &AddArgs) -> Result<Self> {
        Ok(Self {
            command: Self::parse_value(&args.value)?,
            undo_command: None,
        })
    }

    fn parse_value(value: &Vec<String>) -> Result<String> {
        if value.len() == 1 {
            Ok(shell_words::join(shell_words::split(&value[0])?))
        } else {
            Ok(shell_words::join(value))
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.command)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    fn testcase(input: Vec<&str>, output: &str) -> Result<()> {
        assert_eq!(
            Command::parse_value(&input.into_iter().map(|s| s.to_string()).collect())?,
            output,
        );
        Ok(())
    }

    #[test]
    fn test_parse_value() -> Result<()> {
        testcase(vec!["echo", "one two"], r#"echo 'one two'"#)?;
        testcase(vec!["echo", "'one two'"], r#"echo ''\''one two'\'''"#)?;
        testcase(vec!["echo", r#""one two""#], r#"echo '"one two"'"#)?;
        testcase(vec!["echo 'one two'"], r#"echo 'one two'"#)?;
        testcase(vec!["echo"], r#"echo"#)?;
        testcase(vec!["echo one"], r#"echo one"#)?;
        testcase(vec!["'echo one'"], r#"'echo one'"#)?;

        Ok(())
    }
}
