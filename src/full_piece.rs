use serde::{Deserialize, Serialize};
use crate::machine::Machine;
use crate::pieces::PieceEnum;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FullPiece {
    piece: PieceEnum,
    done_on: Vec<Machine>,
    undo: bool,
    undone_on: Option<Vec<Machine>>,
}

pub(crate) enum Todo {
    NO,
    DO,
    UNDO,
}

impl FullPiece {
    fn todo(&self, machine: &Machine) -> Todo {
        let done = self.done_on.contains(machine);
        let undo = self.undo;
        let undone = if undo {
            // SAFETY: if `undo && undone_on.is_none` the configuration is in an illegal state
            Some(self.undone_on.as_ref().unwrap().contains(machine))
        } else {
            // SAFETY: if `!undo && undo_on.is_some` the configuration is in an illegal state
            assert!(self.undone_on.is_some());
            None
        };

        match (done, undo, undone) {
            (false, false, _) => Todo::DO, // Not done, not to undo: DO (undone is `None`)
            (false, true, _) => Todo::NO, // Not done, but to undo: NO (undone must be `Some(false)`)
            (true, false, _) => Todo::NO, // Done, not to undo: NO (undone is `None`)
            (true, true, Some(false)) => Todo::UNDO, // Done, but to undo, and not undone yet: UNDO
            (true, true, Some(true)) => Todo::NO, // Done, but to undo, but already undone: NO
            (_, true, None) => unreachable!(), // SAFETY: We just accounted for this
        }
    }
}
