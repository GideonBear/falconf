use std::collections::HashMap;
use crate::full_piece::FullPiece;
use crate::machine::{Machine, MachineData};

struct Data {
    pieces: Vec<FullPiece>,
    machines: HashMap<Machine, MachineData>,
}
