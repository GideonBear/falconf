use crate::cli::TopLevelArgs;
use crate::cli::machines::MachineRef;
use crate::execution_context::ExecutionContext;
use crate::installation::Installation;
use crate::machine::machines_by_name;
use color_eyre::Result;
use color_eyre::eyre::eyre;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Specify the machine. 'this' is a shortcut for the current machine.
    machine: MachineRef,

    /// Change the name (which is the hostname by default) of the machine.
    #[arg(long)]
    name: Option<String>,
}

pub fn edit(top_level_args: TopLevelArgs, mut args: Args) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let ctx = ExecutionContext::new(&installation, &top_level_args)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let machines = data.machines_mut();

    if let Some(name) = &args.name
        && machines_by_name(machines).contains_key(name.as_str())
    {
        return Err(eyre!("Another machine already has name '{name}'"));
    }

    let (_machine, machine_data) = args.machine.resolve_mut(machines, &ctx)?;

    if let Some(name) = args.name.take() {
        machine_data.name = name;
    }

    // Push changes
    repo.write_and_push(vec![], None)?;

    Ok(())
}
