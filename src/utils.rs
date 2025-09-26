use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use std::io::Write;
use std::path::Path;
use std::{fs, io};

// pub fn if_sudo(program: &str, sudo: bool) -> process::Command {
//     if sudo {
//         let mut cmd = process::Command::new("sudo");
//         cmd.arg(program);
//         cmd
//     } else {
//         process::Command::new(program)
//     }
// }

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
        print!("{question} (y/n) ");
        io::stdout().flush()?;
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

pub fn create_parent(path: &Path) -> Result<()> {
    let parent = path.parent().ok_or_eyre("File doesn't have parent")?;
    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn remove_empty_dirs(path: &Path) -> Result<()> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                remove_empty_dirs(&p)?;
                if fs::read_dir(&p)
                    .map(|mut d| d.next().is_none())
                    .unwrap_or(false)
                {
                    fs::remove_dir(&p)?;
                }
            }
        }
    }
    Ok(())
}
