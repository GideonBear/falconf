use crate::full_piece::FullPiece;
use crate::machine::{Machine, MachineData};
use crate::migrations;
use color_eyre::Result;
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
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

    pub const fn machines(&self) -> &IndexMap<Machine, MachineData> {
        &self.machines
    }

    pub const fn machines_mut(&mut self) -> &mut IndexMap<Machine, MachineData> {
        &mut self.machines
    }

    pub const fn as_mut_parts(
        &mut self,
    ) -> (
        &mut IndexMap<u32, FullPiece>,
        &mut IndexMap<Machine, MachineData>,
    ) {
        (&mut self.pieces, &mut self.machines)
    }
}

pub fn from_reader<T: DeserializeOwned>(reader: impl Read) -> Result<T> {
    Ok(ron::de::from_reader(reader)?)
}

pub fn from_file<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    from_reader(reader)
}

pub fn to_writer<T: Serialize>(data: &T, writer: impl Write) -> Result<()> {
    Ok(ron::Options::default().to_io_writer_pretty(
        writer,
        data,
        ron::ser::PrettyConfig::default(),
    )?)
}

pub fn to_file<T: Serialize>(data: &T, path: &Path) -> Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    to_writer(data, writer)
}
