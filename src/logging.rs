use color_eyre::Result;
use std::fmt::Display;
use std::process::{Command, ExitStatus};

pub trait CommandExt {
    fn status_checked(&mut self) -> Result<ExitStatus>;
}

impl CommandExt for Command {
    fn status_checked(&mut self) -> Result<ExitStatus> {
        log(format!("Executing: {self:?}"))?;
        command_error::CommandExt::status_checked(self).map_err(Into::into)
    }
}

fn log(message: impl Display) -> Result<()> {
    // TODO: do actual logging instead of printing. Deny printing lint?
    //  Logging should also print, but also log to file
    println!("{message}");
    Ok(())
}

// TODO: log everywhere!
