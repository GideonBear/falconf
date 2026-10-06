use crate::full_piece::FullPiece;
use crate::machine::{Machine, MachineData};
use crate::migrations;
use color_eyre::Result;
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write as _};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    version: u32,
    pieces: IndexMap<u32, FullPiece>,
    machines: IndexMap<Machine, MachineData>,
}

impl Data {
    pub fn init_new() -> Self {
        Self {
            version: migrations::VERSION,
            pieces: IndexMap::new(),
            machines: IndexMap::new(),
        }
    }

    pub const fn pieces(&self) -> &IndexMap<u32, FullPiece> {
        &self.pieces
    }

    pub const fn pieces_mut(&mut self) -> &mut IndexMap<u32, FullPiece> {
        &mut self.pieces
    }

    pub const fn machines_mut(&mut self) -> &mut IndexMap<Machine, MachineData> {
        &mut self.machines
    }
}

pub fn from_file<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data = ron::de::from_reader(reader)?;
    Ok(data)
}

pub fn to_file<T: Serialize>(data: &T, path: &Path) -> Result<()> {
    let file = File::create(path)?;
    let string = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())?;
    let mut writer = BufWriter::new(file);
    writer.write_all(string.as_bytes())?;
    Ok(())
}
