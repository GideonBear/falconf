use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use color_eyre::owo_colors::OwoColorize;
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

#[expect(clippy::print_stdout)]
pub fn press_enter() -> io::Result<()> {
    println!("Press Enter to continue...");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}

#[expect(clippy::print_stdout)]
pub fn confirm(question: &str) -> io::Result<bool> {
    loop {
        print!("{question} (y/n) ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let ans = input.to_ascii_lowercase();
        let ans = ans.trim_end();
        if ans == "y" || ans == "yes" {
            return Ok(true);
        } else if ans == "n" || ans == "no" {
            return Ok(false);
        } else {
            println!("Invalid answer.");
        }
    }
}

#[expect(clippy::print_stdout)]
pub fn prompt(question: &str) -> io::Result<String> {
    print!("{question}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim_end().to_string())
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

pub fn print_id(id: u32) -> String {
    let id = format!("[{id:08x}]");
    let id = id.magenta();
    let id = id.bold();
    format!("{id}")
}
