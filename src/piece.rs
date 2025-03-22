use std::path::PathBuf;
use std::{io, process};
use std::fs::remove_file;
use std::process::ExitStatus;

enum ExecutionError {
    ProcessError(ExitStatus),
    IoError(io::Error),
}

impl From<io::Error> for ExecutionError {
    fn from(err: io::Error) -> ExecutionError {
        ExecutionError::IoError(err)
    }
}

type ExecutionResult = Result<(), ExecutionError>;

trait ExitStatusExt {
    fn to_execution_result(self) -> ExecutionResult;
}

impl ExitStatusExt for ExitStatus {
    fn to_execution_result(self) -> ExecutionResult {
        if self.success() {
            Ok(())
        } else {
            Err(ExecutionError::ProcessError(self))
        }
    }
}

trait ResultExitStatusExt {
    fn to_execution_result(self) -> ExecutionResult;
}

impl ResultExitStatusExt for io::Result<ExitStatus> {
    fn to_execution_result(self) -> ExecutionResult {
        self
            .map_err(|e| ExecutionError::from(e))?
            .to_execution_result()
    }
}

fn if_sudo(program: &str, sudo: bool) -> process::Command {
    if sudo {
        let mut cmd = process::Command::new("sudo");
        cmd.arg(program);
        cmd
    } else {
        process::Command::new(program)
    }
}

/// A single piece of configuration
trait Piece: Sized {
    /// Execute the piece
    fn execute(&self)-> ExecutionResult;

    /// Execute multiple of these pieces in bulk. Returns None when this Piece does not support it.
    fn execute_bulk(_pieces: &[&Self]) -> Option<ExecutionResult> {
        None
    }

    /// Undo the piece. Returns None when the undo is user-defined and has not been defined.
    fn undo(&self) -> Option<ExecutionResult>;

    /// Undo multiple of these pieces in bulk. Returns None when this Piece does not support it.
    fn undo_bulk(_pieces: &[&Self]) -> Option<Option<ExecutionResult>> {
        None
    }
}

/// Sym/hardlink a file
struct File {
    location: PathBuf,
    /// If the file should be a hardlink or symlink
    hardlink: bool,
    /// If the file should be created as sudo
    sudo: bool,
}

impl Piece for File {
    fn execute(&self) -> ExecutionResult {
        let repo_file = find_file(&self.location);

        let mut cmd = if_sudo("ln", self.sudo)
            .arg(&repo_file)
            .arg(&self.location);
        if !self.hardlink {
            cmd.arg("--symbolic");
        }

        cmd
            .status()
            .to_execution_result()
    }

    fn undo(&self) -> Option<ExecutionResult> {
        Some(
            remove_file(&self.location)
                .map_err(|e| ExecutionError::from(e))
        )
    }
}

/// Run an arbitrary command with bash
struct Command {
    /// The command to run
    command: String,
    /// If the command should be run with sudo
    sudo: bool,
    /// The command to run when undoing
    undo_command: Option<String>,
}

impl Piece for Command {
    fn execute(&self) -> ExecutionResult {
        Self::run_command(&self.command, self.sudo)
    }

    fn undo(&self) -> Option<ExecutionResult> {
        // This will return None if self.undo_command is None
        self.undo_command.as_ref().map(|cmd| Self::run_command(cmd, self.sudo))
    }
}

impl Command {
    fn run_command(command: &str, sudo: bool) -> ExecutionResult {
        if_sudo("bash", sudo)
            .arg("-c")
            .arg(command)
            .status()
            .to_execution_result()
    }
}

/// Request the user to perform an action manually *sad robot face*
struct Manual {
    /// The message to show the user
    message: String,
}

impl Piece for Manual {
    fn execute(&self) -> ExecutionResult {
        Self::print_message(&self.message);
        Ok(())
    }

    fn undo(&self) -> Option<ExecutionResult> {
        Self::print_message(&format!("UNDO the following change: {}", self.message));
        Some(Ok(()))
    }
}

impl Manual {
    fn print_message(message: &str) {
        println!("Manual action required");
        println!("{}", message);
        println!("Continue when the action is performed.");
        press_enter();
    }
}

/// Install a package with apt
struct AptPackage {
    /// The package to install
    package: String,
}

impl Piece for AptPackage {
    fn execute(&self) -> ExecutionResult {
        // Safety: we implement the method below and know it will only return Some
        Self::execute_bulk(&[self]).unwrap()
    }

    fn execute_bulk(pieces: &[&Self]) -> Option<ExecutionResult> {
        Some(Self::apt_command("install", pieces))
    }

    fn undo(&self) -> Option<ExecutionResult> {
        // Safety: we implement the method below and know it will only return Some
        Self::undo_bulk(&[self]).unwrap()
    }

    fn undo_bulk(pieces: &[&Self]) -> Option<Option<ExecutionResult>> {
        Some(Some(Self::apt_command("remove", pieces)))
    }
}

impl AptPackage {
    fn apt_command(command: &str, pieces: &[&Self]) -> ExecutionResult {
        process::Command::new("apt")
            .arg(command)
            .args(pieces.iter().map(|p| &p.package))
            .status()
            .to_execution_result()
    }
}
