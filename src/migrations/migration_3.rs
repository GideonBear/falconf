use crate::group::Group;
use crate::machine::{Machine, MachineData};
use crate::migrations::LitU32;
use crate::pieces::PieceEnum;
use derive_into_2::Convert;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Deserialize)]
struct OldFullPiece {
    piece: PieceEnum,
    comment: Option<String>,
    done_on: Vec<Machine>,
    undone_on: Option<Vec<Machine>>,
    one_time_todo_on: Option<Vec<Machine>>,
}

#[derive(Debug, Clone, Serialize)]
struct NewFullPiece {
    piece: PieceEnum,
    comment: Option<String>,
    done_on: BTreeSet<Machine>,
    group: Group,
}

impl From<OldFullPiece> for NewFullPiece {
    fn from(old: OldFullPiece) -> Self {
        // Was never written or used so should be None
        assert!(old.one_time_todo_on.is_none());

        let undone = old.undone_on.is_some();

        let mut done_on: BTreeSet<Machine> = old.done_on.into_iter().collect();
        for undone_on in old.undone_on.into_iter().flatten() {
            assert!(done_on.remove(&undone_on));
        }

        Self {
            piece: old.piece,
            comment: old.comment,
            done_on,
            group: if undone { Group::None } else { Group::All },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Convert)]
#[convert(into(path = "NewData"))]
pub(super) struct OldData {
    #[convert(default)]
    #[allow(unused)]
    version: LitU32<2>,
    pieces: IndexMap<u32, OldFullPiece>,
    machines: IndexMap<Machine, MachineData>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct NewData {
    version: LitU32<3>,
    pieces: IndexMap<u32, NewFullPiece>,
    machines: IndexMap<Machine, MachineData>,
}
