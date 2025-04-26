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
    use crate::testing::TestRemote;
    use log::LevelFilter;
    use tempdir::TempDir;

    #[test]
    fn test_init() -> Result<()> {
        color_eyre::install().ok();

        env_logger::Builder::new()
            .filter_level(LevelFilter::Debug)
            .init();

        let remote = TestRemote::new()?;

        // New
        {
            let temp = TempDir::new("test_falconf")?;
            let falconf_path = temp.path().join(".falconf");

            let top_level_args = TopLevelArgs {
                log_level: "".to_string(),
                verbose: false,
                path: falconf_path,
            };

            let args = InitArgs {
                new: true,
                remote: remote.address().to_string(),
            };

            println!("Initting new repository...");
            init(top_level_args, args)?;
        }

        // Existing
        {
            let temp = TempDir::new("test_falconf")?;
            let falconf_path = temp.path().join(".falconf");

            let top_level_args = TopLevelArgs {
                log_level: "".to_string(),
                verbose: false,
                path: falconf_path,
            };

            let args = InitArgs {
                new: false,
                remote: remote.address().to_string(),
            };

            println!("Initting existing repository...");
            init(top_level_args, args)?;
        }

        Ok(())
    }
}
