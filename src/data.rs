use crate::full_piece::FullPiece;
use crate::machine::{Machine, MachineData};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    pieces: Vec<FullPiece>,
    machines: HashMap<Machine, MachineData>,
}

impl Data {
    pub fn init_new() -> Self {
        Self {
            pieces: vec![],
            machines: HashMap::new(),
        }
    }

    pub fn pieces_mut(&mut self) -> &mut Vec<FullPiece> {
        &mut self.pieces
    }

    pub fn machines(&self) -> Vec<&Machine> {
        self.machines.keys().collect()
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader)?;
        Ok(data)
    }

    pub fn to_file(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }
}
