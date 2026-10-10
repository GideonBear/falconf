use crate::cli::TopLevelArgs;
use crate::execution_context::ExecutionContext;
use crate::machine::{Machine, MachineData};
use crate::utils::match_prefix;
use clap::Subcommand;
use color_eyre::Result;
use color_eyre::eyre::Context;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

mod edit;
mod list;

#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Show information about machines")]
    List(list::Args),

    #[command(about = "Edit a machine")]
    Edit(edit::Args),
}

#[derive(Debug, Clone)]
pub enum MachineRef {
    This,
    NameOrId(String),
}

impl MachineRef {
    fn _resolve(
        self,
        machines: &IndexMap<Machine, MachineData>,
        ctx: &ExecutionContext,
    ) -> Result<Machine> {
        match self {
            Self::This => Ok(ctx.machine),
            Self::NameOrId(s) => {
                let mut candidates: HashMap<String, Machine> = HashMap::new();
                for (&machine, machine_data) in machines {
                    candidates.insert(machine.0.to_string(), machine);
                    candidates.insert(machine_data.name.clone(), machine);
                }
                let found = match_prefix(candidates.keys().map(|s| s.as_str()), &s)
                    .wrap_err("Couldn't match with a machine name or id")?;
                Ok(candidates[found])
            }
        }
    }

    #[expect(unused)]
    fn resolve<'a>(
        self,
        machines: &'a IndexMap<Machine, MachineData>,
        ctx: &ExecutionContext,
    ) -> Result<(Machine, &'a MachineData)> {
        let machine = self._resolve(machines, ctx)?;
        Ok((
            machine,
            #[expect(clippy::missing_panics_doc, reason = "illegal configuration")]
            machines
                .get(&machine)
                // If returned by the NameOrId arm, it was retrieved directly from `machines`
                .expect("This machine not in data file"),
        ))
    }

    fn resolve_mut<'a>(
        self,
        machines: &'a mut IndexMap<Machine, MachineData>,
        ctx: &ExecutionContext,
    ) -> Result<(Machine, &'a mut MachineData)> {
        let machine = self._resolve(machines, ctx)?;
        Ok((
            machine,
            #[expect(clippy::missing_panics_doc, reason = "illegal configuration")]
            machines
                .get_mut(&machine)
                // If returned by the NameOrId arm, it was retrieved directly from `machines`
                .expect("This machine not in data file"),
        ))
    }
}

impl FromStr for MachineRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "this" {
            return Ok(Self::This);
        }
        Ok(Self::NameOrId(s.to_string()))
    }
}

pub fn machines(top_level: TopLevelArgs, args: Args) -> Result<()> {
    let Args { command } = args;
    let command = command.unwrap_or_else(|| Commands::List(list::Args::default()));
    match command {
        Commands::List(args) => list::list(top_level, args, &mut io::stdout().lock()),
        Commands::Edit(args) => edit::edit(top_level, args),
    }
}
