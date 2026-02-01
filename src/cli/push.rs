use crate::cli::TopLevelArgs;
use crate::installation::Installation;
use crate::utils::confirm;
use clap::Args;
use color_eyre::eyre::{Context, Result, eyre};
use git2::DiffFormat;
use log::info;
use std::path::Path;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct PushArgs {}

#[allow(clippy::print_stdout)]
pub fn push(top_level_args: TopLevelArgs, _args: PushArgs) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    installation.pull_and_read(true)?;
    let repo = installation.repo_mut();

    // Get the diff
    let diff = repo.diff_index_to_workdir()?;

    // Get the changed files
    let files: Vec<PathBuf> = diff
        .deltas()
        .filter_map(|d| d.new_file().path())
        .map(|path| {
            path.strip_prefix("files")
                .wrap_err("A file not in files/ was changed in the repo")
                .map(Path::to_path_buf)
        })
        .collect::<Result<_, _>>()?;

    // If there are no changes, exit
    if files.is_empty() {
        info!("Repo is clean, there are no changes to commit");
        return Ok(());
    }

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

    if !confirm("The above diff will be committed. Do you want to continue?")? {
        return Err(eyre!("Aborted"));
    }

    // Push changes
    repo.write_and_push(files)?;

    Ok(())
}

// TODO(test): add tests
// #[cfg(test)]
// mod tests {
//     #![allow(clippy::missing_panics_doc)]
//
//     use super::*;
// }
