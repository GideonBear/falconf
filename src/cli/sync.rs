use crate::cli::TopLevelArgs;
use crate::execution_context::ExecutionContext;
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use color_eyre::Result;
use log::info;

#[derive(clap::Args, Debug)]
pub struct Args {}

#[allow(clippy::needless_pass_by_value)]
pub fn sync(top_level_args: TopLevelArgs, _args: Args) -> Result<()> {
    let mut installation = Installation::get_no_check_synced(&top_level_args)?;
    let ctx = ExecutionContext::new(&installation, &top_level_args)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();

    // Do out-of-sync (todo) changes
    if let Err(err) = FullPiece::do_todo(data.pieces_mut(), &ctx) {
        info!("Found error during sync; writing and pushing the changes that *were* done");
        repo.write_and_push(vec![], None)?;
        return Err(err);
    }

    // Push changes
    repo.write_and_push(vec![], None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::add::tests::{add_util, add_util_no_test_run};
    use crate::cli::remove::remove;
    use crate::cli::undo::tests::undo_util;
    use crate::cli::{add, remove};
    use crate::testing::{TestEnv, get_piece};
    use color_eyre::eyre::OptionExt;
    use log::debug;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_sync() -> Result<()> {
        let mut env = TestEnv::new(2)?;
        let test_1 = env.fs.join("test_1.txt");

        env.with(0, |f| {
            File::create(&test_1)?.write_all(b"test_1_content")?;
            debug!("Created {test_1:?}");
            let test_1_s = test_1.to_str().ok_or_eyre("Invalid path")?.to_string();
            add_util_no_test_run(f, add::Piece::File, vec![test_1_s.clone()])?;
            // Testing the order. This should execute after the file is added.
            add_util(
                f,
                add::Piece::Command,
                vec![format!("test -L '{}'", test_1_s)],
            )?;

            assert!(test_1.exists());
            assert!(test_1.is_symlink());
            assert_eq!(std::fs::read_to_string(&test_1)?, "test_1_content");
            Ok(())
        })?;

        env.with(1, |f| {
            // Remove with no args; this is for manual testing of check_synced
            let top_level_args = TopLevelArgs::new_testing(f.clone(), false);
            remove(
                top_level_args,
                remove::Args {
                    pieces: vec![],
                    force: false,
                },
            )?;
            let top_level_args = TopLevelArgs::new_testing(f.clone(), false);
            let args = Args {};
            sync(top_level_args, args)?;

            // After syncing, the file is created
            assert!(test_1.exists());
            assert!(test_1.is_symlink());
            assert_eq!(std::fs::read_to_string(&test_1)?, "test_1_content");
            Ok(())
        })?;

        env.with(0, |f| {
            // Explicitly do not pull machine 0 here to test auto-pulling

            undo_util(f, get_piece(f, 1)?)?;
            // Is a test run
            assert!(test_1.exists());
            assert!(test_1.is_symlink());
            Ok(())
        })?;

        env.with(1, |f| {
            let top_level_args = TopLevelArgs::new_testing(f.clone(), false);
            let args = Args {};
            sync(top_level_args, args)?;

            assert!(!test_1.exists());
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_atomic() -> Result<()> {
        let mut env = TestEnv::new(2)?;

        env.with(0, |f| {
            // Always succeeds
            add_util(f, add::Piece::Command, vec![String::from("true")])?;
            // Always fails
            add_util(f, add::Piece::Command, vec![String::from("false")])?;
            Ok(())
        })?;

        env.with(1, |f| {
            let top_level_args = TopLevelArgs::new_testing(f.clone(), false);
            assert!(sync(top_level_args, Args {}).is_err());

            let top_level_args = TopLevelArgs::new_testing(f.clone(), true);
            let installation = Installation::get(&top_level_args)?;
            // The first one (true) was successfully marked as done
            assert!(
                installation
                    .repo()
                    .data()
                    .pieces()
                    .get_index(1)
                    .ok_or_eyre("Cannot find added piece")?
                    .1
                    .done_on()
                    .contains(&installation.machine())
            );
            // The second one (false) wasn't marked as done
            assert!(
                !installation
                    .repo()
                    .data()
                    .pieces()
                    .get_index(2)
                    .ok_or_eyre("Cannot find added piece")?
                    .1
                    .done_on()
                    .contains(&installation.machine())
            );
            Ok(())
        })?;

        Ok(())
    }
}
