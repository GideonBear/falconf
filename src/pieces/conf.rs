use crate::cli::add;
use crate::execution_context::ExecutionContext;
use crate::piece::{BulkPiece, Piece};
use crate::utils::confirm;
use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use ini_preserve::Ini;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Conf {
    file: PathBuf,
    section: String,
    key: String,
    value: String,
    original_value: Option<String>,
}

impl Piece for Conf {
    fn seed(&self, _ctx: &ExecutionContext) -> Result<(String, bool, Option<String>)> {
        Err(eyre!(
            "conf pieces do not support seeding. Place them after the init piece."
        ))
    }
}

impl BulkPiece for Conf {
    fn execute_bulk(pieces: Vec<&mut Self>, _ctx: &ExecutionContext) -> Result<()> {
        Self::edit(pieces, Self::execute)
    }

    fn undo_bulk(pieces: Vec<&mut Self>, _ctx: &ExecutionContext) -> Result<()> {
        Self::edit(pieces, Self::undo)
    }
}

impl Conf {
    fn execute(&mut self, ini: &mut Ini) -> Result<()> {
        let current_value = ini.get(&self.section, &self.key);
        let original_value = self.original_value.as_deref();
        if current_value != original_value
            && !confirm(&format!(
                "conf key [{}] {} value differs from recorded original value. Is {}, expected {}. Overwrite with piece value ({}) anyway?",
                self.section,
                self.key,
                current_value.unwrap_or("<missing>"),
                original_value.unwrap_or("<missing>"),
                self.value
            ))?
        {
            return Err(eyre!("Aborted"));
        }

        ini.set(&self.section, &self.key, &self.value);
        Ok(())
    }

    fn undo(&mut self, ini: &mut Ini) -> Result<()> {
        let current_value = ini.get(&self.section, &self.key);
        if current_value != Some(&self.value)
            && !confirm(&format!(
                "conf key [{}] {} value differs from piece value. Is {}, expected {}. Overwrite anyway with original ({}) anyway?",
                self.section,
                self.key,
                current_value.unwrap_or("<missing>"),
                self.value,
                self.original_value.as_deref().unwrap_or("<missing>")
            ))?
        {
            return Err(eyre!("Aborted"));
        }

        if let Some(original_value) = &self.original_value {
            ini.set(&self.section, &self.key, original_value);
        } else {
            ini.remove(&self.section, &self.key);
        }
        Ok(())
    }

    fn edit(pieces: Vec<&mut Self>, cb: impl Fn(&mut Self, &mut Ini) -> Result<()>) -> Result<()> {
        let by_file = pieces
            .into_iter()
            .into_group_map_by(|piece| piece.file.clone());
        for (file, pieces) in by_file {
            let mut ini = Ini::load(&file)
                .map_err(|s| eyre!(s))
                .wrap_err("Failed to parse ini file")?;

            for piece in pieces {
                cb(piece, &mut ini)?;
            }

            // TODO: unwritable files
            ini.save(&file)
                .map_err(|s| eyre!(s))
                .wrap_err("Failed to write ini file")?;
        }
        Ok(())
    }

    fn new(file: PathBuf, section: String, key: String, value: String) -> Result<Self> {
        let original_value = if file.exists() {
            Ini::load(&file)
                .map_err(|s| eyre!(s))
                .wrap_err("Failed to parse ini file")?
                .get(&section, &key)
                .map(|x| x.to_string())
        } else {
            None
        };

        Ok(Self {
            file,
            section,
            key,
            value,
            original_value,
        })
    }

    #[allow(unused)]
    pub fn new_with_original(
        file: PathBuf,
        section: String,
        key: String,
        value: String,
        original_value: Option<String>,
    ) -> Self {
        Self {
            file,
            section,
            key,
            value,
            original_value,
        }
    }

    pub fn from_cli(args: &add::Args) -> Result<Self> {
        if let [file, section, key, value] = &args.value[..] {
            Ok(Self::new(
                file.parse()?,
                section.to_string(),
                key.to_string(),
                value.to_string(),
            )?)
        } else {
            Err(eyre!(
                "Invalid format for conf piece: expected <file> <schema> <key> <value>"
            ))
        }
    }
}

impl Display for Conf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "In {}: [{}] {}={}",
            self.file.display(),
            self.section,
            self.key,
            self.value
        )
    }
}
