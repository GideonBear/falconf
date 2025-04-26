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

#[allow(clippy::print_stdout)]
pub fn press_enter() -> io::Result<()> {
    println!("Press Enter to continue...");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}

pub fn set_eq<T: Eq>(vec1: &[T], vec2: &[T]) -> bool {
    vec1.len() == vec2.len() && vec1.iter().all(|x| vec2.contains(x))
}
