use crate::cli::{PieceRef, TopLevelArgs};
use crate::execution_context::ExecutionContext;
use crate::installation::Installation;
use color_eyre::Result;
use color_eyre::eyre::OptionExt;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Specify piece ids. '-' is a shortcut for the last piece.
    #[clap(required = true)]
    pub(crate) pieces: Vec<PieceRef>,
}

#[allow(clippy::needless_pass_by_value)]
pub fn done(top_level_args: TopLevelArgs, args: Args) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let ctx = ExecutionContext::new(&installation, &top_level_args)?;
    installation.pull_and_read(true, &ctx)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let pieces = data.pieces_mut();

    let to_done: Vec<_> = args
        .pieces
        .into_iter()
        .map(|piece| {
            let id = piece.resolve(pieces)?;
            pieces.get(&id).ok_or_eyre("Piece not found")?;
            Ok(id)
        })
        .collect::<Result<_>>()?;

    for piece in to_done {
        #[expect(clippy::missing_panics_doc, reason = "Checked above")]
        pieces.get_mut(&piece).unwrap().done(ctx.machine);
    }

    // Push changes
    repo.write_and_push(vec![], None)?;

    Ok(())
}
