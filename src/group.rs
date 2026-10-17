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
    pub fn parse_for_piece(s: &str, machines: &IndexMap<Machine, MachineData>) -> Result<Self> {
        match s {
            "all" => Ok(Self::All),
            "none" => Ok(Self::None),
            machine_name if let Some(&machine) = machines_by_name(machines).get(machine_name) => {
                Ok(Self::Machine(machine))
            }
            s => {
                let group = Self::Custom(s.to_string());
                if group.machines(machines).next().is_none() {
                    return Err(eyre!(
                        "This group has no machines yet. Add this group to a machine first, using `falconf machines edit`."
                    ));
                }
                Ok(group)
            }
        }
    }

    pub fn parse_for_machine(s: &str, machines: &IndexMap<Machine, MachineData>) -> Result<String> {
        match s {
            "all" | "none" => Err(eyre!(format!(
                "Cannot use reserved '{s}' group for a machine"
            ))),
            machine_name if machines_by_name(machines).contains_key(machine_name) => {
                Err(eyre!(format!(
                    "Cannot use machine name '{s}' as a group, it is already implicitly a group containing that machine"
                )))
            }
            custom => Ok(custom.to_string()),
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
