use crate::cli::{SyncArgs, TopLevelArgs};
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use color_eyre::Result;

pub fn sync(top_level_args: TopLevelArgs, _args: SyncArgs) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let machine = *installation.machine();
    let repo = installation.repo_mut();

    // Pull the repo
    repo.pull_and_read()?;

    let data = repo.data_mut();

    // Do out-of-sync (todo) changes
    FullPiece::do_todo(data.pieces_mut().iter_mut().collect(), &machine)?;

    // Push changes
    repo.write_and_push()?;

    Ok(())
}
