use crate::pieces::apt_package::AptPackage;
use crate::pieces::command::Command;
use crate::pieces::file::File;
use crate::pieces::manual::Manual;

mod apt_package;
mod command;
mod file;
mod manual;

#[non_exhaustive]
pub(crate) enum PieceEnum {
    AptPackage(AptPackage),
    Command(Command),
    File(File),
    Manual(Manual),
}
