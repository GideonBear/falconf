use crate::migrations::{LitU32, RawValue};
use derive_into_2::Convert;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Convert)]
#[convert(into(path = "NewPieceEnum"))]
enum OldPieceEnum {
    Bulk(RawValue),
    NonBulk(RawValue),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
enum NewPieceEnum {
    Bulk(RawValue),
    NonBulk(RawValue),
}

#[derive(Debug, Clone, Deserialize, Convert)]
#[convert(into(path = "NewFullPiece"))]
struct OldFullPiece {
    piece: OldPieceEnum,
    comment: RawValue,
    done_on: RawValue,
    group: RawValue,
}

#[derive(Debug, Clone, Serialize)]
struct NewFullPiece {
    piece: NewPieceEnum,
    comment: RawValue,
    done_on: RawValue,
    group: RawValue,
}

#[derive(Debug, Clone, Deserialize, Convert)]
#[convert(into(path = "NewData"))]
pub(super) struct OldData {
    #[convert(default)]
    #[expect(unused)]
    version: LitU32<4>,
    pieces: IndexMap<RawValue, OldFullPiece>,
    machines: RawValue,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct NewData {
    version: LitU32<5>,
    pieces: IndexMap<RawValue, NewFullPiece>,
    machines: RawValue,
}
