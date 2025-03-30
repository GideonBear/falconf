use std::fmt::Display;
use std::io;
use std::process::Command;

pub trait CommandExt {
    fn log_execution(&mut self) -> io::Result<&mut Self>;
}

impl CommandExt for Command {
    fn log_execution(&mut self) -> io::Result<&mut Self> {
        log(format!("{self:?}"))?;
        Ok(self)
    }
}

fn log(message: impl Display) -> io::Result<()> {
    // TODO: do actual logging instead of printing. Deny printing lint?
    //  Logging should also print, but also log to file
    println!("{}", message);
    Ok(())
}

// TODO: log everywhere!
