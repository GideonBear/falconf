use crate::full_piece::FullPiece;
use crate::machine::{Machine, MachineData};
use crate::migrations::LitU32;
use derive_into::Convert;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Convert)]
#[convert(into(path = "MachineData"))]
struct OldMachineData {
    #[convert(rename = "name")]
    pub hostname: String,
}

#[derive(Debug, Clone, Deserialize, Convert)]
#[convert(into(path = "NewData"))]
pub(super) struct OldData {
    #[convert(default)]
    #[allow(unused)]
    version: LitU32<1>,
    pieces: IndexMap<u32, FullPiece>,
    machines: IndexMap<Machine, OldMachineData>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct NewData {
    version: LitU32<2>,
    pieces: IndexMap<u32, FullPiece>,
    machines: IndexMap<Machine, MachineData>,
}
