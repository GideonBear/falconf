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

fn main() -> Result<(), eyre::Report> {
    color_eyre::config::HookBuilder::new()
        .display_location_section(true)
        .install()?;

    cli::main()
}
