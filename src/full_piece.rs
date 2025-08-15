use crate::cli::AddArgs;
use crate::cli::UndoArgs;
use crate::execution_data::ExecutionData;
use crate::machine::Machine;
use crate::pieces::PieceEnum;
use crate::utils::set_eq;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullPiece {
    piece: PieceEnum,
    /// An optional comment to clarify the use of the piece
    comment: Option<String>,
    /// The machines on which this piece is already done
    done_on: Vec<Machine>,
    /// `Some` if this piece should be undone
    /// The machines on which this piece is already undone
    undone_on: Option<Vec<Machine>>,
    /// `Some` if this piece should be executed just once (so not on new machines)
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
    // TODO: one-time support (in all below methods)
    pub fn new(piece: PieceEnum, comment: Option<String>) -> Self {
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

    pub fn do_todo(
        pieces: Vec<&mut Self>,
        machine: &Machine,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        let mut to_execute = vec![];
        let mut to_undo = vec![];

        for piece in pieces {
            match piece.todo(machine) {
                Todo::Noop => {}
                Todo::Execute => to_execute.push(piece),
                Todo::Undo => to_undo.push(piece),
            }
        }

        PieceEnum::execute_bulk(
            to_execute.iter().map(|x| &x.piece).collect(),
            execution_data,
        )?;
        for piece in to_execute {
            piece.done_on.push(*machine);
        }

        PieceEnum::undo_bulk(to_undo.iter().map(|x| &x.piece).collect(), execution_data)?;
        for piece in to_undo {
            // SAFETY: since we got `Todo::Undo` back we can assume that `piece.undone_one.is_some()`
            #[expect(clippy::missing_panics_doc, reason = "code path")]
            piece.undone_on.as_mut().unwrap().push(*machine);
        }

        Ok(())
    }

    pub fn add(args: &AddArgs, execution_data: &ExecutionData) -> Result<(u32, Self)> {
        let mut piece = Self::from_cli(args)?;

        piece.done_on.push(execution_data.machine);
        if args.not_done_here {
            PieceEnum::execute_bulk(vec![&piece.piece], execution_data)?;
        }

        Ok((Self::new_id(), piece))
    }

    pub fn undo(&mut self, args: &UndoArgs, execution_data: &ExecutionData) -> Result<()> {
        if self.undone_on.is_some() {
            return Err(eyre!("This piece is already undone"));
        }

        self.undone_on = Some(vec![execution_data.machine]);
        if !args.done_here {
            PieceEnum::undo_bulk(vec![&self.piece], execution_data)?;
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

    /// Display information about this piece in the console
    pub fn print<W: Write>(&self, writer: &mut W, id: u32) -> Result<()> {
        let id_prefix = format!("[{id:08x}]");
        let id_prefix = id_prefix.magenta();
        let id_prefix = id_prefix.bold();
        let comment_suffix = if let Some(comment) = &self.comment {
            format!(" // {comment}")
        } else {
            String::new()
        };
        let unused_suffix = if self.unused() { " (unused)" } else { "" };
        let unused_suffix = unused_suffix.italic();
        let unused_suffix = unused_suffix.bright_cyan();
        // TODO: Workaround for https://github.com/owo-colors/owo-colors/issues/45. Fix better.
        if self.undone_on.is_some() {
            write!(
                writer,
                "{}{}{}{}{}",
                id_prefix.strikethrough(),
                " ".strikethrough(),
                self.piece.strikethrough(),
                comment_suffix.strikethrough(),
                unused_suffix,
            )?;
        } else {
            write!(
                writer,
                "{} {}{}{}",
                id_prefix, self.piece, comment_suffix, unused_suffix,
            )?;
        }

        writeln!(writer)?;
        Ok(())
    }
}
