use crate::machine::Machine;
use crate::piece::ExecutionResult;
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
    Noop,
    Execute,
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
            (false, false, _) => Todo::Execute, // Not done, not to undo: Execute (undone is `None`)
            (false, true, _) => Todo::Noop, // Not done, but to undo: Noop (undone must be `Some(false)`)
            (true, false, _) => Todo::Noop, // Done, not to undo: Noop (undone is `None`)
            (true, true, Some(false)) => Todo::Undo, // Done, but to undo, and not undone yet: Undo
            (true, true, Some(true)) => Todo::Noop, // Done, but to undo, but already undone: Noop
            (_, true, None) => unreachable!(), // SAFETY: We just accounted for this
        }
    }

    fn do_todo(pieces: Vec<&mut Self>, machine: &Machine) -> ExecutionResult {
        let mut to_execute = vec![];
        let mut to_undo = vec![];

        for piece in pieces {
            match piece.todo(machine) {
                Todo::Noop => {}
                Todo::Execute => to_execute.push(piece),
                Todo::Undo => to_undo.push(piece),
            }
        }

        PieceEnum::execute_bulk(to_execute.iter().map(|x| &x.piece).collect())?;
        for piece in to_execute {
            piece.done_on.push(machine.clone());
        }

        PieceEnum::undo_bulk(to_undo.iter().map(|x| &x.piece).collect())?;
        for piece in to_undo {
            // SAFETY: since we got `Todo::Undo` back we can assume that `piece.undo == true`,
            //  Thus `undone_on` must be `Some`, or the configuration is illegal.
            assert!(piece.undo);
            piece.undone_on.as_mut().unwrap().push(machine.clone());
        }

        Ok(())
    }
}
