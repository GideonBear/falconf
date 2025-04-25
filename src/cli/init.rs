use crate::cli::{InitArgs, TopLevelArgs};
use crate::installation::Installation;
use color_eyre::Result;

pub fn init(top_level_args: TopLevelArgs, args: InitArgs) -> Result<()> {
    Installation::init(&top_level_args, &args.remote, args.new)?;
    Ok(())
}
