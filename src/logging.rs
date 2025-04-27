use color_eyre::Result;
use log::info;
use std::process::{Command, ExitStatus, Output};

pub trait CommandExt {
    fn status_checked(&mut self) -> Result<ExitStatus>;

    fn output_checked(&mut self) -> Result<Output>;

    fn output_fallible(&mut self) -> Result<Output>;
}

impl CommandExt for Command {
    fn status_checked(&mut self) -> Result<ExitStatus> {
        log_execution(self);
        command_error::CommandExt::status_checked(self).map_err(Into::into)
    }

    fn output_checked(&mut self) -> Result<Output> {
        log_execution(self);
        command_error::CommandExt::output_checked(self).map_err(Into::into)
    }

    fn output_fallible(&mut self) -> Result<Output> {
        log_execution(self);
        self.output().map_err(Into::into)
    }
}

fn log_execution(command: &Command) {
    info!("Executing: `{}`", as_string(command));
}

fn as_string(command: &Command) -> String {
    format!(
        "{} {}",
        command.get_program().to_string_lossy(),
        command
            .get_args()
            .map(|arg| quoted(arg.to_string_lossy().to_string()))
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn quoted(s: String) -> String {
    if s.contains(' ') {
        format!("\"{s}\"")
    } else {
        s
    }
}

// TODO: log everywhere!
