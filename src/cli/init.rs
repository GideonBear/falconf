use crate::cli::{InitArgs, TopLevelArgs};
use crate::installation::Installation;
use color_eyre::Result;
use color_eyre::eyre::WrapErr;

pub fn init(top_level_args: TopLevelArgs, args: InitArgs) -> Result<()> {
    Installation::init(&top_level_args, &args.remote, args.new).wrap_err("Failed to init")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{TempDirSub, TestRemote};
    use log::debug;
    use tempdir::TempDir;

    pub fn init_new(remote: &TestRemote) -> Result<TempDirSub> {
        let temp = TempDir::new("test_falconf")?;
        let falconf_path = temp.path().join(".falconf");

        let top_level_args = TopLevelArgs {
            log_level: "".to_string(),
            verbose: false,
            path: falconf_path.clone(),
        };

        let args = InitArgs {
            new: true,
            remote: remote.address().to_string(),
        };

        debug!("Initting new repository...");
        init(top_level_args, args)?;

        Ok(TempDirSub::new(temp, falconf_path))
    }

    pub fn init_existing(remote: &TestRemote) -> Result<TempDirSub> {
        let temp = TempDir::new("test_falconf")?;
        let falconf_path = temp.path().join(".falconf");

        let top_level_args = TopLevelArgs {
            log_level: "".to_string(),
            verbose: false,
            path: falconf_path.clone(),
        };

        let args = InitArgs {
            new: false,
            remote: remote.address().to_string(),
        };

        debug!("Initting existing repository...");
        init(top_level_args, args)?;

        Ok(TempDirSub::new(temp, falconf_path))
    }

    #[test]
    fn test_init() -> Result<()> {
        let remote = TestRemote::new()?;

        init_new(&remote)?;
        init_existing(&remote)?;

        Ok(())
    }
}
