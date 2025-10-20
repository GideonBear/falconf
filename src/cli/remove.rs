use crate::cli::TopLevelArgs;
use crate::cli::parse_piece_id;
use crate::installation::Installation;
use clap::Args;
use color_eyre::eyre;
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::Result;
use std::fs::remove_file;

// TODO(low): add a command to remove all unused pieces
#[derive(Args, Debug)]
pub struct RemoveArgs {
    #[clap(
        value_parser = parse_piece_id,
        required = true
    )]
    pub(crate) piece_ids: Vec<u32>,

    /// Remove the piece even if it is not unused
    #[arg(long, short)]
    pub force: bool,
}

pub fn remove(top_level_args: TopLevelArgs, args: RemoveArgs) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    installation.pull_and_read(true)?;
    let repo = installation.repo_mut();
    let file_dir = repo.file_dir()?;
    let pieces = repo.data().pieces();

    let pieces_to_remove = args
        .piece_ids
        .iter()
        .map(|piece_id| pieces.get(piece_id).ok_or_eyre("Piece not found"))
        .collect::<Result<Vec<_>>>()?;

    // Check if it's unused
    for piece in &pieces_to_remove {
        if !args.force && !piece.unused() {
            return Err(eyre::eyre!(
                "Piece is still in use. Pass --force to remove it anyway, without undoing."
            ));
        }
    }

    // Remove attached files
    let mut removed_files = vec![];
    for piece in pieces_to_remove {
        if let Some(file) = piece.file() {
            removed_files.push(file.to_path_buf());
            remove_file(file_dir.join(file))?;
        }
    }
    repo.clean_file_dir()?;

    let pieces = repo.data_mut().pieces_mut();

    // Remove the piece
    for piece_id in args.piece_ids {
        pieces.shift_remove(&piece_id);
    }

    // Push changes
    // Not much to fail, we don't need to split writes
    repo.write_and_push(removed_files)?;

    Ok(())
}

// TODO(test): add tests
// #[cfg(test)]
// mod tests {
//     #![allow(clippy::missing_panics_doc)]
//
//     use super::*;
//     use crate::cli;
//     use crate::cli::add::tests::add_util;
//     use crate::cli::init::tests::init_util;
//     use crate::cli::undo::tests::undo_util;
//     use crate::testing::{TestRemote, get_last_piece};
//     use color_eyre::eyre::OptionExt;
//     use log::debug;
//     use std::fs::{File, create_dir_all};
//     use std::io::Write;
//
//     #[test]
//     fn test_remove() -> color_eyre::Result<()> {
//         let remote = TestRemote::new()?;
//
//         let local = init_util(&remote, true)?;
//         add_util(
//             local_1.path(),
//             cli::Piece::Command,
//             vec!["echo hello".to_string()],
//         )?;
//
//         assert!(!test1.exists());
//
//         let local_2 = init_util(&remote, false)?;
//         let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), false);
//         let args = SyncArgs {};
//         sync(top_level_args, args)?;
//
//         debug!("Checking {test1:?}");
//         assert!(test1.exists());
//         assert_eq!(
//             std::fs::read_to_string(temp.path().join("test1.txt"))?,
//             "test1"
//         );
//
//         // Explicitly do not pull local 1 here to test auto-syncing
//
//         undo_util(local_1.path(), get_last_piece(local_1.path())?)?;
//         assert!(test1.exists());
//
//         let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), false);
//         let args = SyncArgs {};
//         sync(top_level_args, args)?;
//
//         assert!(!test1.exists());
//
//         Ok(())
//     }
// }
