use crate::cli::TopLevelArgs;
use crate::cli::add::add_internal;
use crate::full_piece::FullPiece;
use crate::logging::CommandExt;
use crate::pieces::gsettings::Gsettings;
use crate::pieces::{NonBulkPieceEnum, PieceEnum};
use crate::utils::{confirm, press_enter};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use itertools::Itertools;
use log::info;
use std::collections::BTreeMap;
use std::process::Command;

#[derive(clap::Args, Debug)]
pub struct Args {}

#[expect(clippy::print_stdout)]
#[allow(clippy::needless_pass_by_value)]
pub fn watch_gsettings(top_level_args: TopLevelArgs, _args: Args) -> Result<()> {
    fn get_values() -> Result<BTreeMap<(String, String), String>> {
        Command::new("gsettings")
            .arg("list-recursively")
            .output_checked_utf8()?
            .stdout
            .trim()
            .split('\n')
            .map(|s| {
                let (schema, key, value) = s
                    .splitn(3, ' ')
                    .collect_tuple()
                    .ok_or_else(|| eyre!("Line did not have three items: {s}"))?;
                Ok(((schema.to_string(), key.to_string()), value.to_string()))
            })
            .collect::<Result<_>>()
    }

    let old_values = get_values()?;

    println!("Change settings and press enter when done");
    press_enter()?;

    let new_values = get_values()?;

    let changes: Vec<_> = old_values
        .into_iter()
        .zip(new_values)
        .map(
            |(((old_schema, old_key), old_value), ((new_schema, new_key), new_value))| {
                if old_schema != new_schema || old_key != new_key {
                    Err(eyre!("New schemas or keys were added or removed"))
                } else {
                    Ok(((old_schema, old_key), old_value, new_value))
                }
            },
        )
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|(_, old_value, new_value)| old_value != new_value)
        .collect();

    if changes.is_empty() {
        info!("No changes found.");
        return Ok(());
    }

    let changes: Vec<_> = changes
        .into_iter()
        .filter_map(|((schema, key), old_value, new_value)| {
            println!("{schema}.{key}: {old_value} -> {new_value}");
            match confirm("Do you want to add this change?") {
                Err(e) => Some(Err(eyre!(e))),
                Ok(false) => None,
                Ok(true) => Some(Ok(((schema, key), old_value, new_value))),
            }
        })
        .collect::<Result<_>>()?;

    if changes.is_empty() {
        info!("All changes denied.");
        return Ok(());
    }

    add_internal(
        &top_level_args,
        changes
            .into_iter()
            .map(|((schema, key), old_value, new_value)| {
                Ok((
                    FullPiece::new(
                        PieceEnum::NonBulk(NonBulkPieceEnum::Gsettings(
                            Gsettings::new_with_original(schema, key, new_value, old_value)?,
                        )),
                        None,
                    ),
                    None,
                    true,
                ))
            })
            .collect::<Result<_>>()?,
    )?;

    Ok(())
}
