use crate::cli::TopLevelArgs;
use crate::cli::machines::MachineRef;
use crate::execution_context::ExecutionContext;
use crate::group::Group;
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

    /// Add a group that include this machine
    #[arg(long)]
    group: Vec<String>,

    /// Remove group from this machine
    #[arg(long)]
    remove_group: Vec<String>,
}

impl Args {
    #[cfg(test)]
    pub fn new_testing(machine: MachineRef, group: String) -> Self {
        Self {
            machine,
            name: None,
            group: vec![group],
            remove_group: vec![],
        }
    }
}

pub fn edit(top_level_args: TopLevelArgs, mut args: Args) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let ctx = ExecutionContext::new(&installation, &top_level_args)?;
    let repo = installation.repo_mut();
    let data = repo.data_mut();
    let machines = data.machines_mut();

    let (_machine, machine_data) = args.machine.resolve(machines, &ctx)?;

    if let Some(name) = &args.name
        && machines_by_name(machines).contains_key(name.as_str())
    {
        return Err(eyre!("Another machine already has name '{name}'"));
    }

    let add_groups: Vec<_> = args
        .group
        .iter()
        .map(|group| Group::parse_for_machine(group, machines))
        .collect::<Result<_>>()?;

    let remove_groups: Vec<_> = args
        .remove_group
        .iter()
        .map(|group| {
            if !machine_data.groups.contains(group) {
                Err(eyre!("That machine isn't part of '{group}'"))
            } else {
                Ok(group)
            }
        })
        .collect::<Result<_>>()?;

    if add_groups.iter().any(|x| remove_groups.contains(&x)) {
        return Err(eyre!(
            "Cannot add and remove a group from a machine at the same time"
        ));
    }

    let (_machine, machine_data) = args.machine.resolve_mut(machines, &ctx)?;

    if let Some(name) = args.name.take() {
        machine_data.name = name;
    }
    for group in add_groups {
        machine_data.groups.insert(group);
    }
    for group in remove_groups {
        machine_data.groups.remove(group);
    }

    // Push changes
    repo.write_and_push(vec![], None)?;

    Ok(())
}
