use crate::full_piece::FullPiece;
use crate::machine::{Machine, MachineData};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Data {
    pieces: Vec<FullPiece>,
    machines: HashMap<Machine, MachineData>,
}

impl Data {
    pub(crate) fn new() -> Self {
        Self {
            pieces: vec![],
            machines: HashMap::new(),
        }
    }

    pub(crate) fn from_file(path: &Path) -> Result<Self, DataError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader)?;
        Ok(data)
    }

    pub(crate) fn to_file(&self, path: &Path) -> Result<(), DataError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }
}

#[derive(Debug)]
enum DataError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl From<std::io::Error> for DataError {
    fn from(e: std::io::Error) -> Self {
        DataError::Io(e)
    }
}

impl From<serde_json::Error> for DataError {
    fn from(e: serde_json::Error) -> Self {
        DataError::Json(e)
    }
}
