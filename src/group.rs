use crate::machine::{Machine, MachineData, machines_by_name};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Group {
    All,
    None,
    Machine(Machine),
    Custom(String),
}

impl Group {
    #[expect(unused)] // TODO
    pub fn parse_for_piece(s: &str, machines: &IndexMap<Machine, MachineData>) -> Self {
        match s {
            "all" => Self::All,
            "none" => Self::None,
            machine_name if let Some(&machine) = machines_by_name(machines).get(machine_name) => {
                Self::Machine(machine)
            }
            s => Self::Custom(s.to_string()),
        }
    }

    #[expect(unused)] // TODO
    pub fn parse_for_machine(s: &str, machines: &IndexMap<Machine, MachineData>) -> Result<Self> {
        match Self::parse_for_piece(s, machines) {
            Self::All | Self::None => Err(eyre!(format!(
                "Cannot use reserved '{s}' group for a machine"
            ))),
            Self::Machine(_) => Err(eyre!(format!(
                "Cannot use machine name '{s}' as a group, it is already implicitly a group containing that machine"
            ))),
            Self::Custom(custom) => Ok(Self::Custom(custom)),
        }
    }

    pub fn machines<'a>(
        &self,
        machines: &'a IndexMap<Machine, MachineData>,
    ) -> impl Iterator<Item = (&'a Machine, &'a MachineData)> {
        machines
            .iter()
            .filter(|&(machine, machine_data)| machine_data.in_group(*machine, self))
    }
}
