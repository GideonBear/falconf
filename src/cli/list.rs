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

    // TODO: some kind of piece identifier
    for piece in pieces {
        piece.print(writer)?;
    }
    writeln!(writer)?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    #![allow(clippy::missing_panics_doc)]

    use super::*;
    use crate::cli;
    use crate::cli::add::tests::add_util;
    use crate::cli::init::tests::init_util;
    use crate::testing::TestRemote;
    use color_eyre::Result;
    use std::io;

    #[test]
    fn test_list() -> Result<()> {
        let remote = TestRemote::new()?;
        let local = init_util(&remote, true)?;

        add_util(local.path(), cli::Piece::Apt, vec![String::from("htop")])?;

        // TODO: test comments
        // TODO: test all piece types

        let top_level_args = TopLevelArgs::new_testing(local.path().clone());
        let args = ListArgs {};
        let mut writer = io::Cursor::new(vec![]);

        list(top_level_args, args, &mut writer)?;

        assert_eq!(
            String::from_utf8(writer.into_inner())?,
            String::from(
                r#"apt install htop
"#
            )
        );

        Ok(())
    }
}
