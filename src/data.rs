use crate::full_piece::FullPiece;
use crate::machine::{Machine, MachineData};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    pieces: BTreeMap<u32, FullPiece>,
    machines: HashMap<Machine, MachineData>,
}

impl Data {
    pub fn init_new() -> Self {
        Self {
            pieces: BTreeMap::new(),
            machines: HashMap::new(),
        }
    }

    pub fn pieces(&self) -> &BTreeMap<u32, FullPiece> {
        &self.pieces
    }

    pub fn pieces_mut(&mut self) -> &mut BTreeMap<u32, FullPiece> {
        &mut self.pieces
    }

    pub fn machines(&self) -> Vec<&Machine> {
        self.machines.keys().collect()
    }

    pub fn machines_mut(&mut self) -> &mut HashMap<Machine, MachineData> {
        &mut self.machines
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data = ron::de::from_reader(reader)?;
        Ok(data)
    }

    pub fn to_file(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let string = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(string.as_bytes())?;
        Ok(())
    }
}
