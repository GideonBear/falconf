use crate::full_piece::FullPiece;
use crate::machine::{Machine, MachineData};
use std::collections::HashMap;

struct Data {
    pieces: Vec<FullPiece>,
    machines: HashMap<Machine, MachineData>,
}
