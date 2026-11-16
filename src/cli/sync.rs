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
pub mod tests {
    use super::*;
    use crate::cli::add::tests::{add_util, add_util_no_test_run};
    use crate::cli::push::push;
    use crate::cli::push::tests::push_util;
    use crate::cli::remove::remove;
    use crate::cli::undo::tests::undo_util;
    use crate::cli::{add, push, remove};
    use crate::testing::{TestEnv, get_piece};
    use crate::testing::{git_status, setup_two_files};
    use color_eyre::eyre::OptionExt;
    use command_error::CommandExt as _;
    use log::debug;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::process::Command;

    pub fn sync_util(f: &Path) -> Result<()> {
        sync(TopLevelArgs::new_testing(f.to_path_buf(), false), Args {})
    }

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
                    remove_with_changes: false,
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

    #[test]
    fn test_changes_to_files_not_overwritten() -> Result<()> {
        let mut env = TestEnv::new(2)?;
        let test_1 = env.fs.join("test_1.txt");
        let test_1_s = test_1.to_str().ok_or_eyre("Invalid path")?.to_string();

        env.with(0, |f| {
            File::create(&test_1)?.write_all(b"test_1_content")?;
            debug!("Created {test_1:?}");
            add_util_no_test_run(f, add::Piece::File, vec![test_1_s.clone()])?;
            Ok(())
        })?;

        env.with(1, |f| {
            let top_level_args = TopLevelArgs::new_testing(f.clone(), false);
            let args = Args {};
            sync(top_level_args, args)?;
            File::create(&test_1)?.write_all(b"test_1_content_edit_local_2")?;
            let top_level_args = TopLevelArgs::new_testing(f.clone(), false);
            push(top_level_args, push::Args { yes: true })?;
            Ok(())
        })?;

        env.with(0, |f| {
            File::create(&test_1)?.write_all(b"test_1_content_edit_local_1")?;
            let top_level_args = TopLevelArgs::new_testing(f.clone(), false);
            let args = Args {};
            assert!(sync(top_level_args, args).is_err());
            assert_eq!(
                fs::read_to_string(&test_1)?,
                "test_1_content_edit_local_1".to_string()
            );

            File::create(&test_1)?.write_all(b"test_1_content")?;
            let top_level_args = TopLevelArgs::new_testing(f.clone(), false);
            let args = Args {};
            sync(top_level_args, args)?;
            assert_eq!(
                fs::read_to_string(&test_1)?,
                "test_1_content_edit_local_2".to_string()
            );

            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_pull_keeps_unrelated_local_changes() -> Result<()> {
        let (mut env, a, b) = setup_two_files()?;

        env.with(1, |f| {
            fs::write(&a, "a_1")?;
            push_util(f)
        })?;

        env.with(0, |f| {
            fs::write(&b, "b_0")?;
            sync_util(f)?;
            assert_eq!(fs::read_to_string(&a)?, "a_1");
            assert_eq!(fs::read_to_string(&b)?, "b_0");
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_pull_merges_unpushed_commits() -> Result<()> {
        let (mut env, a, b) = setup_two_files()?;

        env.with(1, |f| {
            fs::write(&a, "a_1")?;
            push_util(f)
        })?;

        env.with(0, |f| {
            // Simulate a commit that failed to push
            fs::write(&b, "b_0")?;
            Command::new("git")
                .arg("-C")
                .arg(f.join("repository"))
                .args(["commit", "--all", "--message", "unpushed"])
                .status_checked()?;
            sync_util(f)?;
            assert_eq!(fs::read_to_string(&a)?, "a_1");
            assert_eq!(fs::read_to_string(&b)?, "b_0");
            Ok(())
        })?;

        env.with(1, |f| {
            sync_util(f)?;
            assert_eq!(fs::read_to_string(&b)?, "b_0");
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_remove_refuses_with_changes() -> Result<()> {
        let (mut env, a, _b) = setup_two_files()?;

        env.with(0, |f| {
            fs::write(&a, "a_0")?;
            let remove_util = |remove_with_changes| {
                remove(
                    TopLevelArgs::new_testing(f.clone(), false),
                    remove::Args {
                        pieces: vec![get_piece(f, 1)?],
                        force: true,
                        remove_with_changes,
                    },
                )
            };
            assert!(remove_util(false).is_err());
            assert_eq!(fs::read_to_string(&a)?, "a_0");
            remove_util(true)?;
            assert!(!fs::exists(fs::read_link(&a)?)?);
            Ok(())
        })?;

        Ok(())
    }

    fn set_push_url(f: &Path, url: &str) -> Result<()> {
        Command::new("git")
            .arg("-C")
            .arg(f.join("repository"))
            .args(["remote", "set-url", "--push", "origin", url])
            .status_checked()?;
        Ok(())
    }

    #[test]
    fn test_pull_merges_data_after_failed_push() -> Result<()> {
        let mut env = TestEnv::new(2)?;
        let remote = env.remote.address.clone();

        env.with(0, |f| {
            add_util(f, add::Piece::Command, vec![String::from("true")])
        })?;

        env.with(1, |f| {
            // Marks the piece as done on machine 1, but fails to push that
            set_push_url(f, "file:///nonexistent")?;
            assert!(sync_util(f).is_err());
            set_push_url(f, &remote)?;
            Ok(())
        })?;

        env.with(0, |f| {
            add_util(f, add::Piece::Command, vec![String::from("echo")])
        })?;

        env.with(1, |f| {
            let installation = Installation::get(&TopLevelArgs::new_testing(f.clone(), true))?;
            let pieces = installation.repo().data().pieces();
            assert_eq!(pieces.len(), 3);
            assert!(pieces[1].done_on().contains(&installation.machine()));
            Ok(())
        })?;

        env.with(0, |f| {
            let installation = Installation::get(&TopLevelArgs::new_testing(f.clone(), true))?;
            let pieces = installation.repo().data().pieces();
            assert_eq!(pieces.len(), 3);
            assert_eq!(pieces[1].done_on().len(), 2);
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_pull_data_conflict_after_failed_push() -> Result<()> {
        let mut env = TestEnv::new(2)?;
        let remote = env.remote.address.clone();

        env.with(1, |f| {
            set_push_url(f, "file:///nonexistent")?;
            assert!(add_util(f, add::Piece::Command, vec![String::from("true")]).is_err());
            set_push_url(f, &remote)?;
            Ok(())
        })?;

        env.with(0, |f| {
            add_util(f, add::Piece::Command, vec![String::from("echo")])
        })?;

        env.with(1, |f| {
            let head = || -> Result<String> {
                Ok(Command::new("git")
                    .arg("-C")
                    .arg(f.join("repository"))
                    .args(["rev-parse", "HEAD"])
                    .output_checked_utf8()?
                    .stdout)
            };
            let head_before = head()?;
            // Both machines added a piece at the end, which git can't merge
            let err = Installation::get(&TopLevelArgs::new_testing(f.clone(), true)).unwrap_err();
            assert!(format!("{err:?}").contains("conflict with remote changes in: data.ron"));
            // Nothing was changed; the unpushed commit is still there
            assert_eq!(head()?, head_before);
            assert_eq!(git_status(f)?, "");
            Ok(())
        })?;

        Ok(())
    }

    // TODO(med): merge data.ron semantically instead of with git, so concurrent additions merge:
    //  when only data.ron conflicts, three-way merge the `pieces` and `machines` maps by key
    #[test]
    #[should_panic(expected = "conflict with remote changes in: data.ron")]
    fn test_pull_merges_concurrent_additions() {
        let inner = || -> Result<()> {
            let mut env = TestEnv::new(2)?;
            let remote = env.remote.address.clone();

            env.with(1, |f| {
                set_push_url(f, "file:///nonexistent")?;
                assert!(add_util(f, add::Piece::Command, vec![String::from("true")]).is_err());
                set_push_url(f, &remote)?;
                Ok(())
            })?;

            env.with(0, |f| {
                add_util(f, add::Piece::Command, vec![String::from("echo")])
            })?;

            env.with(1, |f| {
                let installation = Installation::get(&TopLevelArgs::new_testing(f.clone(), true))?;
                assert_eq!(installation.repo().data().pieces().len(), 3);
                Ok(())
            })
        };
        inner().map_err(|err| format!("{err:?}")).unwrap();
    }
}
