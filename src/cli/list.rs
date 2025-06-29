use crate::cli::{ListArgs, TopLevelArgs};
use crate::installation::Installation;
use std::io::Write;

pub fn list<W: Write>(
    top_level_args: TopLevelArgs,
    _args: ListArgs,
    writer: &mut W,
) -> color_eyre::Result<()> {
    let installation = Installation::get(&top_level_args)?;
    let repo = installation.repo();
    let data = repo.data();
    let pieces = data.pieces();

    for (id, piece) in pieces {
        piece.print(writer, *id)?;
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::cli;
    use crate::cli::add::tests::{add_util, add_util_comment};
    use crate::cli::init::tests::init_util;
    use crate::testing::TestRemote;
    use assert_matches_regex::assert_matches_regex;
    use color_eyre::Result;
    use std::io;
    use tempdir::TempDir;

    #[test]
    fn test_list() -> Result<()> {
        let remote = TestRemote::new()?;
        let local = init_util(&remote, true)?;

        add_util(local.path(), cli::Piece::Apt, vec![String::from("htop")])?;
        add_util(
            local.path(),
            cli::Piece::Command,
            vec![String::from("echo"), String::from("some text")],
        )?;
        let temp = TempDir::new("test_falconf_files")?;
        let test1 = temp.path().join("test1.txt");
        add_util(
            local.path(),
            cli::Piece::File,
            vec![test1.display().to_string()],
        )?;
        add_util(
            local.path(),
            cli::Piece::Manual,
            vec![String::from("some"), String::from("message")],
        )?;
        add_util_comment(local.path(), cli::Piece::Apt, vec!["htop"], "This is a comment!")

        // TODO: test comments

        let top_level_args = TopLevelArgs::new_testing(local.path().clone());
        let args = ListArgs {};
        let mut writer = io::Cursor::new(vec![]);

        list(top_level_args, args, &mut writer)?;

        static ID_RE: &str = r#"\[[0-9a-f]{8}\]"#;

        assert_matches_regex!(
            format!("\n{}", String::from_utf8(writer.into_inner())?),
            format!(
                r#"
{ID_RE} apt install htop
{ID_RE} echo 'some text'
{ID_RE} Tracking file at: {}
{ID_RE} Manual action: some message
"#,
                test1.display()
            )
        );

        Ok(())
    }
}
