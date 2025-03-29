use crate::pieces::apt_package::AptPackage;
use crate::pieces::command::Command;
use crate::pieces::file::File;
use crate::pieces::manual::Manual;
use serde::{Deserialize, Serialize};

mod apt_package;
mod command;
mod file;
mod manual;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PieceEnum {
    AptPackage(AptPackage),
    Command(Command),
    File(File),
    Manual(Manual),
}
