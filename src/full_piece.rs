use crate::cli::AddArgs;
use crate::cli::UndoArgs;
use crate::execution_data::ExecutionData;
use crate::machine::Machine;
use crate::pieces::{NonBulkPieceEnum, PieceEnum};
use crate::utils::{print_id, set_eq};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use indexmap::IndexMap;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullPiece {
    pub piece: PieceEnum,
    /// An optional comment to clarify the use of the piece
    pub comment: Option<String>,
    /// The machines on which this piece is already done
    done_on: Vec<Machine>,
    /// `Some` if this piece should be undone
    /// The machines on which this piece is already undone
    undone_on: Option<Vec<Machine>>,
    /// `Some` if this piece should be executed just once (so not on new machines)
    /// The machines to do it on if `one_time` is true
    one_time_todo_on: Option<Vec<Machine>>,
}

#[derive(Debug, Clone)]
pub enum Todo {
    Noop,
    Execute,
    Undo,
}

type IdPiecePair<'a> = (u32, &'a mut FullPiece);

impl FullPiece {
    // TODO(low): one-time support (in all below methods), and then in cli
    pub const fn new(piece: PieceEnum, comment: Option<String>) -> Self {
        Self {
            piece,
            comment,
            done_on: vec![],
            undone_on: None,
            one_time_todo_on: None,
        }
    }

    fn todo(&self, machine: &Machine) -> Todo {
        let done = self.done_on.contains(machine);
        // `Some` if undo, contains `true` if it was undone on this machine
        let undone = self
            .undone_on
            .as_ref()
            .map(|undone_on| undone_on.contains(machine));

        #[allow(clippy::match_same_arms)]
        match (done, undone) {
            (false, None) => Todo::Execute,     // Not done, not to undo: Execute
            (false, Some(false)) => Todo::Noop, // Not done, but to undo: Noop
            #[expect(clippy::missing_panics_doc, reason = "illegal configuration")]
            #[expect(clippy::panic, reason = "illegal configuration")]
            (false, Some(true)) => panic!("illegal configuration"), // SAFETY: bad config; not done, but also undone
            (true, None) => Todo::Noop,        // Done, not to undo: Noop
            (true, Some(false)) => Todo::Undo, // Done, but to undo, and not undone yet: Undo
            (true, Some(true)) => Todo::Noop,  // Done, but to undo, but already undone: Noop
        }
    }

    pub fn get_todo<'a>(
        pieces: &'a mut IndexMap<u32, Self>,
        machine: &Machine,
    ) -> (Vec<IdPiecePair<'a>>, Vec<IdPiecePair<'a>>) {
        let mut to_execute = vec![];
        let mut to_undo = vec![];

        for (&id, piece) in pieces {
            match piece.todo(machine) {
                Todo::Noop => {}
                Todo::Execute => to_execute.push((id, piece)),
                Todo::Undo => to_undo.push((id, piece)),
            }
        }

        (to_execute, to_undo)
    }

    pub fn do_todo(
        pieces: &mut IndexMap<u32, Self>,
        machine: &Machine,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        let (mut to_execute, mut to_undo) = Self::get_todo(pieces, machine);

        PieceEnum::execute_bulk(
            to_execute
                .iter_mut()
                .map(|(id, x)| {
                    (*id, &mut x.piece, || {
                        x.done_on.push(*machine);
                    })
                })
                .collect(),
            execution_data,
        )?;

        PieceEnum::undo_bulk(
            to_undo
                .iter_mut()
                .map(|(id, x)| {
                    (*id, &mut x.piece, || {
                        // SAFETY: since we got `Todo::Undo` back we can assume that `piece.undone_one.is_some()`
                        #[expect(clippy::missing_panics_doc, reason = "code path")]
                        x.undone_on.as_mut().unwrap().push(*machine);
                    })
                })
                .collect(),
            execution_data,
        )?;

        Ok(())
    }

    pub fn add(args: &AddArgs, execution_data: &ExecutionData) -> Result<(u32, Self)> {
        let mut piece = Self::from_cli(args)?;
        let id = Self::new_id();

        let is_file = piece.file().is_some();

        let mut cb = || {
            piece.done_on.push(execution_data.machine);
        };

        if args.undo.is_some()
            && !matches!(
                piece.piece,
                PieceEnum::NonBulk(NonBulkPieceEnum::Command(_))
            )
        {
            return Err(eyre!(
                "`--undo` only makes sense with a command piece. Autodetected pieces supply their own undo."
            ));
        }

        if args.not_done_here && is_file {
            return Err(eyre!(
                "The concept of '--not-done-here' is incompatible with file pieces. Adding a file piece performs a special action."
            ));
        } else if args.not_done_here || is_file {
            // We could bypass `execute_bulk` here, but this is clearer
            PieceEnum::execute_bulk(vec![(id, &mut piece.piece, cb)], execution_data)?;
        } else {
            // If we don't execute it, just mark it as executed immediately.
            cb();
        }

        Ok((id, piece))
    }

    pub fn undo(&mut self, id: u32, args: &UndoArgs, execution_data: &ExecutionData) -> Result<()> {
        if self.undone_on.is_some() {
            return Err(eyre!("This piece is already undone"));
        }

        let mut cb = || {
            self.undone_on = Some(vec![execution_data.machine]);
        };

        if !args.done_here {
            // We could bypass `execute_bulk` here, but this is clearer
            PieceEnum::undo_bulk(vec![(id, &mut self.piece, cb)], execution_data)?;
        } else {
            // If we don't execute it, just add it immediately.
            cb();
        }

        Ok(())
    }

    /// Returns true if the piece is safe to clean up
    pub fn unused(&self) -> bool {
        if let Some(undone_on) = &self.undone_on {
            // If it's something to undo (whether it's one_time or not),
            //  we don't want to execute it on new machines and can remove it
            //  if none of our existing machines need to have it undone

            set_eq(&self.done_on, undone_on)
        } else if let Some(one_time_todo_on) = &self.one_time_todo_on {
            // We do not want to check with a list of all machines here, since
            //  new machines that are added since the addition of the
            //  one_time piece should not have the piece executed on them.

            set_eq(&self.done_on, one_time_todo_on)
        } else {
            // Any non-undo and non-one_time pieces should never be cleaned up,
            //  since they need to be executed on new machines.

            false
        }
    }

    fn from_cli(args: &AddArgs) -> Result<Self> {
        let comment = args.comment.clone();
        Ok(Self::new(PieceEnum::from_cli(args)?, comment))
    }

    fn new_id() -> u32 {
        rand::rng().next_u32()
    }

    /// Return information about this piece for printing in the console
    pub fn print(&self, id: u32) -> String {
        let id_prefix = print_id(id);

        let undo_suffix = if let PieceEnum::NonBulk(NonBulkPieceEnum::Command(piece)) = &self.piece
            && let Some(undo_command) = &piece.undo_command
        {
            format!(" (undo: {undo_command})")
        } else {
            String::new()
        };
        let undo_suffix = undo_suffix.bright_yellow();

        let comment_suffix = if let Some(comment) = &self.comment {
            format!(" // {comment}")
        } else {
            String::new()
        };

        let unused_suffix = if self.unused() { " (unused)" } else { "" };
        let unused_suffix = unused_suffix.italic();
        let unused_suffix = unused_suffix.bright_cyan();

        // TODO(low): Workaround for https://github.com/owo-colors/owo-colors/issues/45. Fix better.
        if self.undone_on.is_some() {
            format!(
                "{}{}{}{}{}{}",
                id_prefix.strikethrough(),
                " ".strikethrough(),
                self.piece.strikethrough(),
                undo_suffix.strikethrough(),
                comment_suffix.strikethrough(),
                unused_suffix,
            )
        } else {
            format!(
                "{} {}{}{}{}",
                id_prefix, self.piece, undo_suffix, comment_suffix, unused_suffix,
            )
        }
    }

    /// If this is a file piece, get the filename relative to the file dir
    pub fn file(&self) -> Option<&Path> {
        if let PieceEnum::NonBulk(NonBulkPieceEnum::File(file)) = &self.piece {
            Some(file.relative_location())
        } else {
            None
        }
    }

    #[cfg(test)]
    pub const fn done_on(&self) -> &Vec<Machine> {
        &self.done_on
    }
}
