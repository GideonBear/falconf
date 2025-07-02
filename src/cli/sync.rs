use crate::cli::{SyncArgs, TopLevelArgs};
use crate::execution_data::ExecutionData;
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use color_eyre::Result;

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
    repo.write_and_push()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::cli;
    use crate::cli::add::tests::add_util;
    use crate::cli::init::tests::init_util;
    use crate::cli::undo::tests::undo_util;
    use crate::testing::{TestRemote, get_last_piece};
    use color_eyre::eyre::OptionExt;
    use log::debug;
    use std::fs::{File, create_dir_all};
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn test_sync() -> Result<()> {
        let remote = TestRemote::new()?;
        let temp = TempDir::new("test_falconf_files")?;
        let test1 = temp.path().join("test1.txt");

        let local_1 = init_util(&remote, true)?;
        let test1_repository = local_1
            .path()
            .join("repository/files")
            .join(test1.strip_prefix("/")?);
        create_dir_all(
            test1_repository
                .parent()
                .ok_or_eyre("Doesn't have parent")?,
        )?;
        debug!("Created {test1_repository:?}");
        File::create(test1_repository)?.write_all(b"test1")?;
        add_util(
            local_1.path(),
            cli::Piece::File,
            vec![test1.to_str().ok_or_eyre("Invalid path")?.to_string()],
        )?;

        assert!(!test1.exists());

        let local_2 = init_util(&remote, false)?;
        let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), false);
        let args = SyncArgs {};
        sync(top_level_args, args)?;

        debug!("Checking {test1:?}");
        assert!(test1.exists());
        assert_eq!(
            std::fs::read_to_string(temp.path().join("test1.txt"))?,
            "test1"
        );

        let top_level_args = TopLevelArgs::new_testing(local_1.path().clone(), false);
        let args = SyncArgs {};
        sync(top_level_args, args)?;
        assert!(test1.exists());

        undo_util(local_1.path(), get_last_piece(local_1.path())?)?;
        assert!(test1.exists());

        let top_level_args = TopLevelArgs::new_testing(local_2.path().clone(), false);
        let args = SyncArgs {};
        sync(top_level_args, args)?;

        assert!(!test1.exists());

        Ok(())
    }
}
