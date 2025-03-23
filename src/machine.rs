// TODO: contains a unique identifier that is generated on first boot and stored in some config file
#[derive(Debug, Clone)]
pub(crate) struct Machine(());


#[derive(Debug, Clone)]
pub(crate) struct MachineData {
    hostname: String,
}
