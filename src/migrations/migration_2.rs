use crate::migrations::{LitU32, RawValue};
use derive_into_2::Convert;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Convert)]
#[convert(into(path = "NewMachineData"))]
struct OldMachineData {
    #[convert(rename = "name")]
    hostname: String,
}

#[derive(Debug, Clone, Serialize)]
struct NewMachineData {
    name: String,
}

#[derive(Debug, Clone, Deserialize, Convert)]
#[convert(into(path = "NewData"))]
pub(super) struct OldData {
    #[convert(default)]
    #[expect(unused)]
    version: LitU32<1>,
    pieces: RawValue,
    machines: IndexMap<RawValue, OldMachineData>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct NewData {
    version: LitU32<2>,
    pieces: RawValue,
    machines: IndexMap<RawValue, NewMachineData>,
}
