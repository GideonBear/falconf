use crate::machine::Machine;
use crate::piece::ExecutionResult;
use crate::pieces::PieceEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullPiece {
    piece: PieceEnum,
    /// An optional comment to clarify the use of the piece
    comment: Option<String>,
    /// The machines on which this piece is already done
    done_on: Vec<Machine>,
    /// Whether this piece should be undone
    undo: bool,
    /// The machines on which this piece is already undone
    undone_on: Option<Vec<Machine>>,
    /// Whether this piece should be executed just once (so not on new machines)
    one_time: bool,
    /// The machines to do it on if one_time is true
    one_time_todo_on: Option<Vec<Machine>>,
}

#[derive(Debug, Clone)]
pub enum Todo {
    Noop,
    Execute,
    Undo,
}

impl FullPiece {
    // TODO: one-time support
    pub fn new(piece: PieceEnum, comment: Option<String>) -> Self {
        Self {
            piece,
            comment,
            done_on: vec![],
            undo: false,
            undone_on: None,
            one_time: false,
            one_time_todo_on: None,
        }
    }

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

    pub fn do_todo(pieces: Vec<&mut Self>, machine: &Machine) -> ExecutionResult {
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

    /// Returns true if the piece is safe to clean up
    fn unused(&self) -> bool {
        if self.undo {
            // If it's something to undo (whether it's one_time or not),
            //  we don't want to execute it on new machines and can remove it
            //  if none of our existing machines need to have it undone

            // SAFETY: if self.undo self.undo_on must be Some, or the configuration is in an illegal state
            let undone_on = self.undone_on.as_ref().unwrap();
            HashSet::from(&self.done_on) == HashSet::from(undone_on)
        } else if self.one_time {
            // We do not want to check with a list of all machines here, since
            //  new machines that are added since the addition of the
            //  one_time piece should not have the piece executed on them.

            // SAFETY: if self.one_time self.one_time_todo_on must be Some, or the configuration is in an illegal state
            let one_time_todo_on = self.one_time_todo_on.as_ref().unwrap();
            HashSet::from(&self.done_on) == HashSet::from(one_time_todo_on)
        } else {
            // Any non-undo and non-one time pieces should never be cleaned up,
            //  since they need to be executed on new machines.

            false
        }
    }
}
