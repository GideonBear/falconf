use crate::machine::{Machine, MachineData};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Group {
    All,
    None,
    Custom(String),
}

impl Group {
    #[expect(unused)] // TODO
    pub fn parse_for_piece(s: &str) -> Self {
        match s {
            "all" => Self::All,
            "none" => Self::None,
            s => Self::Custom(s.to_string()),
        }
    }

    #[expect(unused)] // TODO
    pub fn parse_for_machine(s: &str) -> Result<Self> {
        match s {
            "all" | "none" => Err(eyre!(format!(
                "Cannot use reserved '{s}' group for a machine"
            ))),
            s => Ok(Self::Custom(s.to_string())),
        }
    }

    pub fn machines<'a>(
        &self,
        machines: &'a IndexMap<Machine, MachineData>,
    ) -> impl Iterator<Item = (&'a Machine, &'a MachineData)> {
        machines
            .iter()
            .filter(|(_machine, machine_data)| machine_data.in_group(self))
    }
}
