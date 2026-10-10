use crate::group::Group;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
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
    pub fn new_this() -> Result<Self> {
        Ok(Self {
            name: hostname::get()?.to_string_lossy().into_owned(),
        })
    }

    /// Return information about this machine for printing in the console
    pub fn print(&self, machine: Machine) -> String {
        format!("{} {}", &machine.0.to_string()[..8], self.name)
    }

    pub fn in_group(&self, group: &Group) -> bool {
        match group {
            Group::All => true,
            Group::None => false,
            Group::Custom(_) => false, // TODO
        }
    }
}
