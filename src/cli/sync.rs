use crate::cli::TopLevelArgs;
use crate::execution_data::ExecutionData;
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use clap::Args;
use color_eyre::Result;

#[derive(Args, Debug)]
pub struct SyncArgs {}

pub fn sync(top_level_args: TopLevelArgs, _args: SyncArgs) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let machine = *installation.machine();
    let execution_data = ExecutionData::new(&installation, &top_level_args)?;
    let repo = installation.repo_mut();

    // Pull the repo
    repo.pull_and_read()?;

    let data = repo.data_mut();

    // Do out-of-sync (todo) changes
    FullPiece::do_todo(
        data.pieces_mut().values_mut().collect(),
        &machine,
        &execution_data,
    )?;

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
    use crate::cli::undo::tests::undo_util;
    use crate::testing::{Position, TestRemote, get_piece};
    use color_eyre::eyre::OptionExt;
    use log::debug;
    use std::fs::{File, create_dir_all, remove_file};
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn test_sync() -> Result<()> {
        let remote = TestRemote::new()?;
        let temp = TempDir::new("test_falconf_files")?;
        let test1 = temp.path().join("test1.txt");

        let local_1 = init_util(&remote, true)?;
        File::create(&test1)?.write_all(b"test1")?;
        debug!("Created {test1:?}");
        let test1_s = test1.to_str().ok_or_eyre("Invalid path")?.to_string();
        add_util_no_test_run(local_1.path(), add::Piece::File, vec![test1_s.clone()])?;
        // Testing the order. This should execute after the file is added.
        add_util(
            local_1.path(),
            add::Piece::Command,
            vec![format!("test -L '{}'", test1_s)],
        )?;

        assert!(test1.is_symlink());

        // Switching to being another machine, the file doesn't exist yet
        remove_file(&test1)?;

        let local_2 = init_util(&remote, false)?;
        let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), false);
        let args = SyncArgs {};
        sync(top_level_args, args)?;

        debug!("Checking {test1:?}");
        assert!(test1.exists());
        assert!(test1.is_symlink());
        assert_eq!(std::fs::read_to_string(&test1)?, "test1");

        // Explicitly do not pull local 1 here to test auto-pulling

        undo_util(
            local_1.path(),
            get_piece(local_1.path(), Position::Index(0))?,
        )?;
        // Is a test run
        assert!(test1.exists());
        assert!(test1.is_symlink());

        let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), false);
        let args = SyncArgs {};
        sync(top_level_args, args)?;

        assert!(!test1.exists());

        Ok(())
    }
}
