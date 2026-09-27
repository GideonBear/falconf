use crate::cli::add;
use crate::execution_data::ExecutionData;
use crate::logging::CommandExt;
use crate::piece::NonBulkPiece;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gsettings {
    schema: String,
    key: String,
    value: String,
    original_value: String,
}

impl NonBulkPiece for Gsettings {
    fn execute(&mut self, _execution_data: &ExecutionData) -> Result<()> {
        Self::set(&self.schema, &self.key, &self.value)
    }

    fn undo(&mut self, _execution_data: &ExecutionData) -> Result<()> {
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
