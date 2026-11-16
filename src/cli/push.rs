use crate::cli::TopLevelArgs;
use crate::installation::Installation;
use crate::repo::Repo;
use crate::utils::confirm;
use color_eyre::eyre::Result;
use git2::DiffFormat;
use log::info;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Push without asking for confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[expect(clippy::print_stdout)]
#[allow(clippy::needless_pass_by_value)]
pub fn push(top_level_args: TopLevelArgs, args: Args) -> Result<()> {
    let repository_path = Installation::get_repository_path_checked(&top_level_args)?;

    // Commit and push before the regular pull, so that local changes can be merged with remote ones
    let pushed = Repo::push_local_changes(&repository_path, |diff| {
        // Print the diff
        // TODO(low): pass this to delta and/or a syntax highlighter
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            match line.origin() {
                '+' | '-' | ' ' => print!("{}", line.origin()),
                _ => {}
            }
            #[expect(
                clippy::missing_panics_doc,
                reason = "Cannot handle the error properly in the closure, and should be utf-8 anyway"
            )]
            let line = str::from_utf8(line.content()).unwrap();
            print!("{line}");
            true
        })?;

        Ok(args.yes || confirm("The above diff will be committed. Do you want to continue?")?)
    })?;

    if !pushed {
        info!("Repo is clean, there are no changes to commit");
    }

    // Pull and do the regular checks
    Installation::get(&top_level_args)?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::cli::sync::tests::sync_util;
    use crate::testing::{git_status, setup_two_files};
    use std::fs;
    use std::path::Path;

    pub fn push_util(f: &Path) -> Result<()> {
        push(
            TopLevelArgs::new_testing(f.to_path_buf(), false),
            Args { yes: true },
        )
    }

    #[test]
    fn test_push_merges_remote_changes() -> Result<()> {
        let (mut env, a, b) = setup_two_files()?;

        env.with(1, |f| {
            fs::write(&a, "a_1")?;
            push_util(f)
        })?;

        env.with(0, |f| {
            fs::write(&b, "b_0")?;
            push_util(f)?;
            assert_eq!(fs::read_to_string(&a)?, "a_1");
            assert_eq!(fs::read_to_string(&b)?, "b_0");
            assert_eq!(git_status(f)?, "");
            Ok(())
        })?;

        env.with(1, |f| {
            sync_util(f)?;
            assert_eq!(fs::read_to_string(&a)?, "a_1");
            assert_eq!(fs::read_to_string(&b)?, "b_0");
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_push_conflict_leaves_changes() -> Result<()> {
        let (mut env, a, _b) = setup_two_files()?;

        env.with(1, |f| {
            fs::write(&a, "a_1")?;
            push_util(f)
        })?;

        env.with(0, |f| {
            fs::write(&a, "a_0")?;
            let status_before = git_status(f)?;
            assert_ne!(status_before, "");
            assert!(push_util(f).is_err());
            // The state is exactly as before
            assert_eq!(fs::read_to_string(&a)?, "a_0");
            assert_eq!(git_status(f)?, status_before);
            // Syncing also fails without changing anything
            assert!(sync_util(f).is_err());
            assert_eq!(fs::read_to_string(&a)?, "a_0");
            assert_eq!(git_status(f)?, status_before);
            Ok(())
        })?;

        Ok(())
    }
}
