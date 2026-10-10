use crate::cli::TopLevelArgs;
use crate::execution_data::ExecutionData;
use crate::installation::Installation;
use color_eyre::Result;
use std::io::Write;

#[derive(clap::Args, Debug, Default)]
pub struct Args {}

pub fn list<W: Write>(top_level_args: TopLevelArgs, _args: Args, writer: &mut W) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let execution_data = ExecutionData::new(&installation, &top_level_args)?;
    installation.pull_and_read(true, &execution_data)?;
    let repo = installation.repo_mut();
    let data = repo.data();
    let machines = data.machines();

    for (machine, machine_data) in machines {
        writeln!(writer, "{}", machine_data.print(*machine))?;
    }

    Ok(())
}
