#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use color_eyre::eyre;

mod cli;
mod data;
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
