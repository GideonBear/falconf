// Warnings are translated to denys in CI
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
#![warn(clippy::panic)]
#![warn(clippy::missing_panics_doc)] // Catches other panics (unwrap, expect)
#![allow(dead_code)] // TODO: remove

use color_eyre::eyre;

mod cli;
mod data;
mod execution_data;
mod full_piece;
mod installation;
mod logging;
mod machine;
mod piece;
mod pieces;
mod repo;
#[cfg(test)]
mod testing;
mod utils;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

const VERSION: &str = built_info::PKG_VERSION;

fn main() -> Result<(), eyre::Report> {
    color_eyre::config::HookBuilder::new()
        .display_location_section(true)
        .install()?;

    cli::main()
}
