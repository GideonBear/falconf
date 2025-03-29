use serde::{Deserialize, Serialize};
use uuid::Uuid;

// TODO: contains a unique identifier that is generated on first boot and stored in some config file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Machine(Uuid);

impl Machine {
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MachineData {
    hostname: String,
}
