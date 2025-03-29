use crate::full_piece::FullPiece;
use crate::machine::{Machine, MachineData};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    pieces: Vec<FullPiece>,
    machines: HashMap<Machine, MachineData>,
}
