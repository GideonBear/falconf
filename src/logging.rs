use std::process::Command;

pub trait CommandExt {
    fn log_execution(&mut self) -> &mut Self;
}

impl CommandExt for Command {
    fn log_execution(&mut self) -> &mut Self {
        // TODO: do actual logging instead of printing. Deny printing lint?
        //  Logging should also print, but also log to file
        println!("{self:?}");
        self
    }
}
