use crate::group::Group;
use crate::utils::OptionExt;
use color_eyre::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Machine(pub Uuid);

impl Machine {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineData {
    pub name: String,
}

impl MachineData {
    pub fn new_this(name: Option<String>) -> Result<Self> {
        Ok(Self {
            name: name
                .unwrap_or_else_fallible(|| Ok(hostname::get()?.to_string_lossy().into_owned()))?,
        })
    }

    /// Return information about this machine for printing in the console
    pub fn print(&self, machine: Machine) -> String {
        format!("{} {}", &machine.0.to_string()[..8], self.name)
    }

    pub fn in_group(&self, machine: Machine, group: &Group) -> bool {
        match group {
            Group::All => true,
            Group::None => false,
            Group::Machine(target) => *target == machine,
            Group::Custom(_) => false, // TODO
        }
    }
}

pub fn machines_by_name(machines: &IndexMap<Machine, MachineData>) -> HashMap<&str, Machine> {
    machines
        .iter()
        .map(|(&machine, machine_data)| (machine_data.name.as_str(), machine))
        .collect()
}
