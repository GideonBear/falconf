use crate::cli::TopLevelArgs;
use crate::installation::Installation;
use clap::Args;
use color_eyre::Result;
use color_eyre::eyre::WrapErr;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Create a new repo instead of cloning an existing one
    #[arg(long, short)]
    new: bool,

    /// The remote url
    remote: String,
}

#[allow(clippy::needless_pass_by_value)]
pub fn init(top_level_args: TopLevelArgs, args: InitArgs) -> Result<()> {
    Installation::init(&top_level_args, &args.remote, args.new).wrap_err("Failed to init")?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::testing::{TempDirSub, TestRemote};
    use log::debug;
    use tempdir::TempDir;

    pub fn init_util(remote: &TestRemote, new: bool) -> Result<TempDirSub> {
        let temp = TempDir::new("test_falconf")?;
        let falconf_path = temp.path().join("test_.falconf_dir");

        let top_level_args = TopLevelArgs::new_testing(falconf_path.clone(), true);

        let args = InitArgs {
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

    // Init is tested more extensively in sync
}
