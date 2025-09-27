use crate::cli::TopLevelArgs;
use crate::installation::Installation;
use clap::Args;
use std::io::Write;

#[derive(Args, Debug)]
pub struct ListArgs {}

pub fn list<W: Write>(
    top_level_args: TopLevelArgs,
    _args: ListArgs,
    writer: &mut W,
) -> color_eyre::Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    installation.pull_and_read(true)?;
    let repo = installation.repo_mut();
    let data = repo.data();
    let pieces = data.pieces();

    for (id, piece) in pieces {
        writeln!(writer, "{}", piece.print(*id))?;
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::cli::add;
    use crate::cli::add::tests::{add_util, add_util_comment};
    use crate::cli::init::tests::init_util;
    use crate::cli::undo::tests::undo_util;
    use crate::testing::{Position, TestRemote, get_piece};
    use color_eyre::Result;
    use log::debug;
    use regex::Regex;
    use std::fs::File;
    use std::io;
    use tempdir::TempDir;

    #[test]
    fn test_list() -> Result<()> {
        let remote = TestRemote::new()?;
        let local = init_util(&remote, true)?;

        // Apt
        add_util(local.path(), add::Piece::Apt, vec![String::from("cowsay")])?;
        // Command
        add_util(
            local.path(),
            add::Piece::Command,
            vec![String::from("echo"), String::from("some text")],
        )?;
        // File
        let temp = TempDir::new("test_falconf_files")?;
        let test1 = temp.path().join("test1.txt");
        File::create(&test1)?.write_all(b"test1")?;
        debug!("Created {test1:?}");
        add_util(
            local.path(),
            add::Piece::File,
            vec![test1.display().to_string()],
        )?;
        // Manual
        add_util(
            local.path(),
            add::Piece::Manual,
            vec![String::from("some"), String::from("message")],
        )?;
        // With comment
        add_util_comment(
            local.path(),
            add::Piece::Apt,
            vec![String::from("cowsay")],
            String::from("This is a comment!"),
        )?;
        // Undone & unused
        add_util(local.path(), add::Piece::Apt, vec![String::from("cowsay")])?;
        let id = get_piece(local.path(), Position::Last)?;
        undo_util(local.path(), id)?;

        let top_level_args = TopLevelArgs::new_testing(local.path().clone(), true);
        let args = ListArgs {};
        let mut writer = io::Cursor::new(vec![]);

        list(top_level_args, args, &mut writer)?;

        let id_re = Regex::new(r"[0-9a-fA-F]{8}").unwrap();
        let output = String::from_utf8(writer.into_inner())?;
        debug!("Got:\n{output}");
        let output = output
            .lines()
            .map(|line| id_re.replace(line, "ID_WAS_HERE").to_string())
            .collect::<Vec<_>>()
            .join("\n");

        let expected = format!(
            "
\u{1b}[1m\u{1b}[35m[ID_WAS_HERE]\u{1b}[39m\u{1b}[0m apt install cowsay\u{1b}[96m\u{1b}[3m\u{1b}[0m\u{1b}[39m
\u{1b}[1m\u{1b}[35m[ID_WAS_HERE]\u{1b}[39m\u{1b}[0m echo 'some text'\u{1b}[96m\u{1b}[3m\u{1b}[0m\u{1b}[39m
\u{1b}[1m\u{1b}[35m[ID_WAS_HERE]\u{1b}[39m\u{1b}[0m Tracking file at: {}\u{1b}[96m\u{1b}[3m\u{1b}[0m\u{1b}[39m
\u{1b}[1m\u{1b}[35m[ID_WAS_HERE]\u{1b}[39m\u{1b}[0m Manual action: some message\u{1b}[96m\u{1b}[3m\u{1b}[0m\u{1b}[39m
\u{1b}[1m\u{1b}[35m[ID_WAS_HERE]\u{1b}[39m\u{1b}[0m apt install cowsay // This is a comment!\u{1b}[96m\u{1b}[3m\u{1b}[0m\u{1b}[39m
\u{1b}[9m\u{1b}[1m\u{1b}[35m[ID_WAS_HERE]\u{1b}[39m\u{1b}[0m\u{1b}[0m\u{1b}[9m \u{1b}[0m\u{1b}[9mapt install cowsay\u{1b}[0m\u{1b}[9m\u{1b}[0m\u{1b}[96m\u{1b}[3m (unused)\u{1b}[0m\u{1b}[39m
",
            test1.display()
        );
        debug!("Expect:\n{}", expected);
        debug!(
            "Got (raw):\n{}",
            format!("{output:?}").trim_matches('"').replace("\\n", "\n")
        );
        debug!(
            "Expect (raw):\n{}",
            format!("{expected:?}")
                .trim_matches('"')
                .replace("\\n", "\n")
        );

        assert_eq!(format!("\n{output}\n"), expected,);

        Ok(())
    }
}
