use crate::full_piece::FullPiece;
use crate::machine::Machine;
use crate::migrations::LitU32;
use derive_into::Convert;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OldAndNewMachineData {
    name: String,
    #[serde(default)]
    groups: BTreeSet<String>,
}

#[derive(Debug, Clone, Deserialize, Convert)]
#[convert(into(path = "NewData"))]
pub(super) struct OldData {
    #[convert(default)]
    #[allow(unused)]
    version: LitU32<3>,
    pieces: IndexMap<u32, FullPiece>,
    machines: IndexMap<Machine, OldAndNewMachineData>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct NewData {
    version: LitU32<4>,
    pieces: IndexMap<u32, FullPiece>,
    machines: IndexMap<Machine, OldAndNewMachineData>,
}
