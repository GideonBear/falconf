use crate::group::Group;
use crate::utils::OptionExt;
use color_eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
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
    // Uses a BTreeSet instead of a HashSet to avoid shuffling when writing data again
    pub groups: BTreeSet<String>,
}

impl MachineData {
    pub fn new_this(name: Option<String>) -> Result<Self> {
        Ok(Self {
            name: name
                .unwrap_or_else_fallible(|| Ok(hostname::get()?.to_string_lossy().into_owned()))?,
            groups: BTreeSet::new(),
        })
    }

    /// Return information about this machine for printing in the console
    pub fn print(&self, machine: Machine) -> String {
        let groups_suffix = if self.groups.is_empty() {
            String::new()
        } else {
            format!(" - {}", self.groups.iter().join(" "))
        };
        let groups_suffix = groups_suffix.bright_green();
        format!(
            "{} {}{groups_suffix}",
            &machine.0.to_string()[..8],
            self.name
        )
    }

    pub fn in_group(&self, machine: Machine, group: &Group) -> bool {
        match group {
            Group::All => true,
            Group::None => false,
            Group::Machine(target) => *target == machine,
            Group::Custom(group) => self.groups.contains(group),
        }
    }
}

pub fn machines_by_name(machines: &IndexMap<Machine, MachineData>) -> HashMap<&str, Machine> {
    machines
        .iter()
        .map(|(&machine, machine_data)| (machine_data.name.as_str(), machine))
        .collect()
}
