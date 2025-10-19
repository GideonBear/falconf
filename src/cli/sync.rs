use crate::cli::TopLevelArgs;
use crate::execution_data::ExecutionData;
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use clap::Args;
use color_eyre::Result;
use log::info;

#[derive(Args, Debug)]
pub struct SyncArgs {}

pub fn sync(top_level_args: TopLevelArgs, _args: SyncArgs) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let machine = *installation.machine();
    let execution_data = ExecutionData::new(&installation, &top_level_args)?;
    installation.pull_and_read(false)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();

    // Do out-of-sync (todo) changes
    if let Err(err) = FullPiece::do_todo(data.pieces_mut(), &machine, &execution_data) {
        info!("Found error during sync; writing and pushing the changes that *were* done");
        repo.write_and_push(vec![])?;
        return Err(err);
    }

    // Push changes
    repo.write_and_push(vec![])?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::cli::add;
    use crate::cli::add::tests::{add_util, add_util_no_test_run};
    use crate::cli::init::tests::init_util;
    use crate::cli::remove::{RemoveArgs, remove};
    use crate::cli::undo::tests::undo_util;
    use crate::testing::{Position, TestRemote, get_piece};
    use color_eyre::eyre::OptionExt;
    use log::debug;
    use std::fs::{File, remove_file};
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn test_sync() -> Result<()> {
        let remote = TestRemote::new()?;
        let temp = TempDir::new("test_falconf_files")?;
        let test_1 = temp.path().join("test_1.txt");
        File::create(&test_1)?.write_all(b"test_1_content")?;

        let local_1 = init_util(&remote, true)?;
        debug!("Created {test_1:?}");
        let test_1_s = test_1.to_str().ok_or_eyre("Invalid path")?.to_string();
        add_util_no_test_run(local_1.path(), add::Piece::File, vec![test_1_s.clone()])?;
        // Testing the order. This should execute after the file is added.
        add_util(
            local_1.path(),
            add::Piece::Command,
            vec![format!("test -L '{}'", test_1_s)],
        )?;

        assert!(test_1.exists());
        assert!(test_1.is_symlink());
        assert_eq!(std::fs::read_to_string(&test_1)?, "test_1_content");

        // Switching to being another machine, the file doesn't exist yet
        remove_file(&test_1)?;
        assert!(!test_1.exists());

        let local_2 = init_util(&remote, false)?;
        // Remove with no args; this is for manual testing of check_synced
        let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), false);
        remove(
            top_level_args,
            RemoveArgs {
                piece_ids: vec![],
                force: false,
            },
        )?;
        let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), false);
        let args = SyncArgs {};
        sync(top_level_args, args)?;

        // After syncing, the file is created
        assert!(test_1.exists());
        assert!(test_1.is_symlink());
        assert_eq!(std::fs::read_to_string(&test_1)?, "test_1_content");

        // Explicitly do not pull local 1 here to test auto-pulling

        undo_util(
            local_1.path(),
            get_piece(local_1.path(), Position::Index(0))?,
        )?;
        // Is a test run
        assert!(test_1.exists());
        assert!(test_1.is_symlink());

        let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), false);
        let args = SyncArgs {};
        sync(top_level_args, args)?;

        assert!(!test_1.exists());

        Ok(())
    }

    #[test]
    fn test_atomic() -> Result<()> {
        let remote = TestRemote::new()?;

        let local_1 = init_util(&remote, true)?;

        // Always succeeds
        add_util(
            local_1.path(),
            add::Piece::Command,
            vec![String::from("true")],
        )?;
        // Always fails
        add_util(
            local_1.path(),
            add::Piece::Command,
            vec![String::from("false")],
        )?;

        let local_2 = init_util(&remote, false)?;
        let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), false);
        assert!(sync(top_level_args, SyncArgs {}).is_err());

        let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), true);
        let installation = Installation::get(&top_level_args)?;
        // The first one (true) was successfully marked as done
        assert!(
            installation
                .repo()
                .data()
                .pieces()
                .get_index(0)
                .ok_or_eyre("Cannot find added piece")?
                .1
                .done_on()
                .contains(installation.machine())
        );
        // The second one (false) wasn't marked as done
        assert!(
            !installation
                .repo()
                .data()
                .pieces()
                .get_index(1)
                .ok_or_eyre("Cannot find added piece")?
                .1
                .done_on()
                .contains(installation.machine())
        );

        Ok(())
    }
}
