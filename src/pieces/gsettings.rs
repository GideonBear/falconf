use crate::cli::add;
use crate::execution_context::ExecutionContext;
use crate::logging::CommandExt;
use crate::piece::{NonBulkPiece, Piece};
use crate::utils::confirm;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use shell_words::quote;
use std::fmt::{Display, Formatter};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Gsettings {
    schema: String,
    key: String,
    value: String,
    original_value: String,
}

impl Piece for Gsettings {
    fn seed(&self, _ctx: &ExecutionContext) -> Result<(String, bool, Option<String>)> {
        Ok((
            format!(
                "gsettings set {} {} {}",
                quote(&self.schema),
                quote(&self.key),
                quote(&self.value)
            ),
            true,
            None,
        ))
    }
}

impl NonBulkPiece for Gsettings {
    fn execute(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        let current_value = Self::get(&self.schema, &self.key)?;
        if current_value != self.original_value
            && !confirm(&format!(
                "gsettings key {}.{} value differs from recorded original value. Is {}, expected {}. Overwrite anyway?",
                self.schema, self.key, current_value, self.original_value
            ))?
        {
            return Err(eyre!("Aborted"));
        }
        Self::set(&self.schema, &self.key, &self.value)
    }

    fn undo(&mut self, _ctx: &ExecutionContext) -> Result<()> {
        Self::set(&self.schema, &self.key, &self.original_value)
    }
}

impl Gsettings {
    fn get(schema: &str, key: &str) -> Result<String> {
        Ok(Command::new("gsettings")
            .arg("get")
            .args([schema, key])
            .output_checked_utf8()?
            .stdout
            .trim()
            .to_string())
    }

    fn set(schema: &str, key: &str, value: &str) -> Result<()> {
        Command::new("gsettings")
            .arg("set")
            .args([schema, key, value])
            .status_checked()?;
        Ok(())
    }

    pub fn new(schema: String, key: String, value: String) -> Result<Self> {
        let original_value = Self::get(&schema, &key)?;

        Ok(Self {
            schema,
            key,
            value,
            original_value,
        })
    }

    pub fn new_with_original(
        schema: String,
        key: String,
        value: String,
        original_value: String,
    ) -> Result<Self> {
        Ok(Self {
            schema,
            key,
            value,
            original_value,
        })
    }

    pub fn from_cli(args: &add::Args) -> Result<Self> {
        if let [schema, key, value] = &args.value[..] {
            Ok(Self::new(
                schema.to_string(),
                key.to_string(),
                value.to_string(),
            )?)
        } else {
            Err(eyre!(
                "Invalid format for gsettings piece: expected <schema> <key> <value>"
            ))
        }
    }
}

impl Display for Gsettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "gsettings set {} {} {}",
            self.schema, self.key, self.value
        )
    }
}
