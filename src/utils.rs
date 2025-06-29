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

#[allow(clippy::print_stdout)]
pub fn confirm(question: &str) -> io::Result<bool> {
    loop {
        println!("{question} (y/n)");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let ans = input.to_ascii_lowercase();
        if ans == "y\n" || ans == "yes\n" {
            return Ok(true);
        } else if ans == "n\n" || ans == "no\n" {
            return Ok(false);
        } else {
            println!("Invalid answer.");
        }
    }
}

pub fn set_eq<T: Eq>(vec1: &[T], vec2: &[T]) -> bool {
    vec1.len() == vec2.len() && vec1.iter().all(|x| vec2.contains(x))
}
