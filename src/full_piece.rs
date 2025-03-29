use crate::machine::Machine;
use crate::pieces::PieceEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullPiece {
    piece: PieceEnum,
    done_on: Vec<Machine>,
    undo: bool,
    undone_on: Option<Vec<Machine>>,
}

#[derive(Debug, Clone)]
pub enum Todo {
    No,
    Do,
    Undo,
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
            (false, false, _) => Todo::Do, // Not done, not to undo: Do (undone is `None`)
            (false, true, _) => Todo::No, // Not done, but to undo: No (undone must be `Some(false)`)
            (true, false, _) => Todo::No, // Done, not to undo: No (undone is `None`)
            (true, true, Some(false)) => Todo::Undo, // Done, but to undo, and not undone yet: Undo
            (true, true, Some(true)) => Todo::No, // Done, but to undo, but already undone: No
            (_, true, None) => unreachable!(), // SAFETY: We just accounted for this
        }
    }
}
