use crate::cli::add;
use crate::execution_data::ExecutionData;
use crate::group::Group;
use crate::machine::{Machine, MachineData};
use crate::pieces::{NonBulkPieceEnum, PieceEnum};
use crate::utils::print_id;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize as _;
use indexmap::IndexMap;
use rand::Rng as _;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct FullPiece {
    pub piece: PieceEnum,
    /// An optional comment to clarify the use of the piece
    pub comment: Option<String>,
    /// The machines on which this piece is done
    // Uses a BTreeSet instead of a HashSet to avoid shuffling when writing data again
    pub done_on: BTreeSet<Machine>,
    /// The group on which this piece should be done
    pub group: Group,
}

#[derive(Debug, Clone)]
enum Todo {
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
            done_on: BTreeSet::new(),
            group: Group::All,
        }
    }

    fn todo(&self, execution_data: &ExecutionData) -> Todo {
        let done = self.done_on.contains(&execution_data.machine);
        let should_do = execution_data.machine_data.in_group(&self.group);

        #[expect(clippy::match_same_arms)]
        match (done, should_do) {
            (false, true) => Todo::Execute,
            (false, false) => Todo::Noop,
            (true, true) => Todo::Noop,
            (true, false) => Todo::Undo,
        }
    }

    pub fn get_todo<'a>(
        pieces: &'a mut IndexMap<u32, Self>,
        execution_data: &ExecutionData,
    ) -> (Vec<IdPiecePair<'a>>, Vec<IdPiecePair<'a>>) {
        let mut to_execute = vec![];
        let mut to_undo = vec![];

        for (&id, piece) in pieces {
            match piece.todo(execution_data) {
                Todo::Noop => {}
                Todo::Execute => to_execute.push((id, piece)),
                Todo::Undo => to_undo.push((id, piece)),
            }
        }

        (to_execute, to_undo)
    }

    pub fn do_todo(pieces: &mut IndexMap<u32, Self>, execution_data: &ExecutionData) -> Result<()> {
        let (mut to_execute, mut to_undo) = Self::get_todo(pieces, execution_data);

        PieceEnum::execute_bulk(
            to_execute
                .iter_mut()
                .map(|(id, x)| {
                    (*id, &mut x.piece, || {
                        x.done_on.insert(execution_data.machine);
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
                        #[expect(
                            clippy::missing_panics_doc,
                            reason = "`todo` only returns `Todo::Undo` if `done_on` contains this machine"
                        )]
                        {
                            assert!(x.done_on.remove(&execution_data.machine));
                        }
                    })
                })
                .collect(),
            execution_data,
        )?;

        Ok(())
    }

    pub fn done(&mut self, machine: Machine) {
        self.done_on.insert(machine);
    }

    pub fn add(&mut self, execution_data: &ExecutionData, done: bool) -> Result<u32> {
        let id = Self::new_id();

        let mut cb = || {
            // Cannot reuse self.done because of partial borrowing
            self.done_on.insert(execution_data.machine);
        };

        if done {
            // If we don't execute it, just mark it as executed immediately.
            cb();
        } else {
            // We could bypass `execute_bulk` here, but this is clearer
            PieceEnum::execute_bulk(vec![(id, &mut self.piece, cb)], execution_data)?;
        }

        Ok(id)
    }

    pub fn undo(&mut self, id: u32, execution_data: &ExecutionData) -> Result<()> {
        if let Group::None = self.group {
            return Err(eyre!("This piece is already undone"));
        }

        let undo_here = self.done_on.contains(&execution_data.machine);

        let mut cb = || {
            // Don't want to assert here; if it doesn't contain it, undo_here is false and we
            //  just want to set the group
            self.done_on.remove(&execution_data.machine);
            self.group = Group::None;
        };

        if undo_here {
            // We could bypass `execute_bulk` here, but this is clearer
            PieceEnum::undo_bulk(vec![(id, &mut self.piece, cb)], execution_data)?;
        } else {
            // If we don't execute it, just add it immediately.
            cb();
        }

        Ok(())
    }

    /// Returns true if the piece is undone or otherwise should be executed on no machines
    // TODO: This includes groups that temporarily have no machines. Is that okay?
    pub fn undone(&self, machines: &IndexMap<Machine, MachineData>) -> bool {
        self.group.machines(machines).next().is_none()
    }

    /// Returns true if the piece is safe to clean up
    pub fn unused(&self, machines: &IndexMap<Machine, MachineData>) -> bool {
        self.undone(machines) && self.done_on.is_empty()
    }

    pub(crate) fn from_cli(args: &add::Args) -> Result<Self> {
        let comment = args.comment.clone();
        Ok(Self::new(PieceEnum::from_cli(args)?, comment))
    }

    fn new_id() -> u32 {
        rand::rng().next_u32()
    }

    /// Return information about this piece for printing in the console
    pub fn print(&self, id: u32, machines: &IndexMap<Machine, MachineData>) -> String {
        let id_prefix = print_id(id);

        let undo_suffix = if let PieceEnum::NonBulk(NonBulkPieceEnum::Command(piece)) = &self.piece
            && let Some(undo_command) = &piece.undo_command
        {
            format!(" (undo: {undo_command})")
        } else {
            String::new()
        };
        let undo_suffix = undo_suffix.bright_yellow();

        let comment_suffix = self
            .comment
            .as_ref()
            .map_or_else(String::new, |comment| format!(" // {comment}"));

        let unused_suffix = if self.unused(machines) {
            " (unused)"
        } else {
            ""
        };
        let unused_suffix = unused_suffix.italic();
        let unused_suffix = unused_suffix.bright_cyan();

        // TODO(low): Workaround for https://github.com/owo-colors/owo-colors/issues/45. Fix better.
        if self.undone(machines) {
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

    pub fn seed(&self, execution_data: &ExecutionData) -> Result<(String, bool, Option<String>)> {
        self.piece.seed(execution_data)
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
    pub const fn done_on(&self) -> &BTreeSet<Machine> {
        &self.done_on
    }
}
