use crate::cli::{InitArgs, TopLevelArgs};
use crate::installation::Installation;
use color_eyre::Result;
use color_eyre::eyre::WrapErr;

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
}
