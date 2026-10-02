use crate::cli::TopLevelArgs;
use crate::execution_data::ExecutionData;
use crate::installation::Installation;
use crate::pieces::{NonBulkPieceEnum, PieceEnum};
use crate::utils::print_id_raw;
use color_eyre::Result;
use std::io::Write;

#[derive(clap::Args, Debug)]
pub struct Args {}

#[allow(clippy::needless_pass_by_value)]
pub fn seed<W: Write>(top_level_args: TopLevelArgs, _args: Args, writer: &mut W) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;
    let execution_data = ExecutionData::new(&installation, &top_level_args)?;
    installation.pull_and_read(true)?;
    let repo = installation.repo_mut();
    let data = repo.data();
    let pieces = data.pieces();

    writeln!(writer, "#!/bin/bash")?;
    writeln!(writer, "set -euxo pipefail")?;

    let mut to_write = String::new();
    for (&id, piece) in pieces {
        let (command, done, post_init) = piece.seed(&execution_data)?;
        writeln!(writer, "{}", command)?;
        writeln!(
            writer,
            "if [ -f ~/.profile ]; then source ~/.profile; fi; if [ -f ~/.bashrc ]; then source ~/.bashrc; fi"
        )?;
        if let Some(post_init) = post_init {
            to_write.push_str(&post_init);
            to_write.push('\n');
        }
        if done {
            to_write.push_str(&format!("falconf done {}", print_id_raw(id)));
            to_write.push('\n');
        }

        if let PieceEnum::NonBulk(NonBulkPieceEnum::FalconfInit(_)) = piece.piece {
            break;
        }
    }
    write!(writer, "{to_write}")?;

    Ok(())
}
