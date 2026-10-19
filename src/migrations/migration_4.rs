use crate::migrations::{LitU32, RawValue};
use derive_into_2::Convert;
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
    #[expect(unused)]
    version: LitU32<3>,
    pieces: RawValue,
    machines: IndexMap<RawValue, OldAndNewMachineData>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct NewData {
    version: LitU32<4>,
    pieces: RawValue,
    machines: IndexMap<RawValue, OldAndNewMachineData>,
}
