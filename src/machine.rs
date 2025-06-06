use color_eyre::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Machine(pub Uuid);

impl Machine {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineData {
    hostname: String,
}

impl MachineData {
    pub fn new_this() -> Result<Self> {
        Ok(Self {
            hostname: hostname::get()?.to_string_lossy().into_owned(),
        })
    }
}
