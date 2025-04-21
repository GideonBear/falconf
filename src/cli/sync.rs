use crate::full_piece::FullPiece;
use crate::installation::Installation;
use crate::piece::ExecutionError;
use crate::repo::PushPullError;

pub fn sync(installation: &mut Installation) -> Result<(), SyncError> {
    let repo = installation.repo();

    // Pull the repo
    repo.pull_and_read()?;

    let data = repo.data();

    // Do out-of-sync (todo) changes
    FullPiece::do_todo(data.pieces(), installation.machine())?;

    // Push changes
    repo.write_and_push()?;

    Ok(())
}

enum SyncError {
    Execution(ExecutionError),
    PushPull(PushPullError),
}

impl From<ExecutionError> for SyncError {
    fn from(value: ExecutionError) -> Self {
        Self::Execution(value)
    }
}

impl From<PushPullError> for SyncError {
    fn from(value: PushPullError) -> Self {
        Self::PushPull(value)
    }
}
