use color_eyre::Result;
use log::info;
use std::process::{Command, ExitStatus, Output};

pub trait CommandExt {
    fn status_checked(&mut self) -> Result<ExitStatus>;

    fn output_checked(&mut self) -> Result<Output>;
}

impl CommandExt for Command {
    fn status_checked(&mut self) -> Result<ExitStatus> {
        info!("Executing: {self:?}");
        command_error::CommandExt::status_checked(self).map_err(Into::into)
    }

    fn output_checked(&mut self) -> Result<Output> {
        info!("Executing: {self:?}");
        command_error::CommandExt::output_checked(self).map_err(Into::into)
    }
}

// TODO: log everywhere!
