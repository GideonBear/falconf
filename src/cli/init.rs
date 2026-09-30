use crate::cli::TopLevelArgs;
use crate::installation::Installation;
use color_eyre::Result;
use color_eyre::eyre::WrapErr as _;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Create a new repo instead of cloning an existing one
    #[arg(long, short)]
    new: bool,

    /// The remote url
    remote: String,
}

#[allow(clippy::needless_pass_by_value)]
pub fn init(top_level_args: TopLevelArgs, args: Args) -> Result<()> {
    let _installation =
        Installation::init(&top_level_args, &args.remote, args.new).wrap_err("Failed to init")?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::full_piece::FullPiece;
    use crate::pieces::falconf_init::FalconfInit;
    use crate::pieces::{NonBulkPieceEnum, PieceEnum};
    use crate::testing::{TempDirSub, TestRemote};
    use log::debug;
    use tempfile::TempDir;

    pub fn init_util(remote: &TestRemote, new: bool) -> Result<TempDirSub> {
        let temp = TempDir::new()?;
        let falconf_path = temp.path().join("test_.falconf_dir");

        let top_level_args = TopLevelArgs::new_testing(falconf_path.clone(), true);

        let args = Args {
            new,
            remote: remote.address().to_string(),
        };

        if new {
            debug!("Initting new repository...");
        } else {
            debug!("Initting existing repository...");
        }
        init(top_level_args, args)?;

        Ok(TempDirSub::new(temp, falconf_path))
    }

    #[test]
    fn test_init() -> Result<()> {
        let remote = TestRemote::new()?;

        init_util(&remote, true)?;
        init_util(&remote, false)?;

        Ok(())
    }

    #[test]
    fn test_init_adds_falconf_init() -> Result<()> {
        let remote = TestRemote::new()?;

        let local = init_util(&remote, true)?;

        let top_level_args = TopLevelArgs::new_testing(local.path().clone(), true);
        let installation = Installation::get(&top_level_args)?;
        let pieces = installation.repo().data().pieces();
        assert_eq!(pieces.len(), 1);
        let mut expected = FullPiece::new(
            PieceEnum::NonBulk(NonBulkPieceEnum::FalconfInit(FalconfInit::new(
                remote.address().to_string(),
            ))),
            None,
        );
        expected.done(installation.machine());
        assert_eq!(pieces[0], expected);

        Ok(())
    }

    // Init is tested more extensively in sync
}
