use crate::cli::{Cli, SyncArgs};
use crate::full_piece::FullPiece;
use crate::installation::Installation;
use color_eyre::Result;

pub fn sync(cli: &Cli, _args: &SyncArgs) -> Result<()> {
    let mut installation = Installation::get(cli)?;
    let machine = *installation.machine();
    let repo = installation.repo();

    // Pull the repo
    repo.pull_and_read()?;

    let data = repo.data();

    // Do out-of-sync (todo) changes
    FullPiece::do_todo(data.pieces().iter_mut().collect(), &machine)?;

    // Push changes
    repo.write_and_push()?;

    Ok(())
}
