use std::{io, process};

pub fn if_sudo(program: &str, sudo: bool) -> process::Command {
    if sudo {
        let mut cmd = process::Command::new("sudo");
        cmd.arg(program);
        cmd
    } else {
        process::Command::new(program)
    }
}

pub fn press_enter() -> io::Result<()> {
    println!("Press Enter to continue...");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}
