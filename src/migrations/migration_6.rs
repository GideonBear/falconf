use crate::migrations::{LitU32, RawValue};
use derive_into_2::Convert;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct OldAndNewFullPiece {
    piece: RawValue,
    comment: RawValue,
    done_on: RawValue,
    group: RawValue,
    #[serde(default)]
    labels: BTreeSet<String>,
}

#[derive(Debug, Clone, Deserialize, Convert)]
#[convert(into(path = "NewData"))]
pub(super) struct OldData {
    #[convert(default)]
    #[expect(unused)]
    version: LitU32<5>,
    pieces: IndexMap<RawValue, OldAndNewFullPiece>,
    machines: RawValue,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct NewData {
    version: LitU32<6>,
    pieces: IndexMap<RawValue, OldAndNewFullPiece>,
    machines: RawValue,
}
