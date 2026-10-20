use crate::cli::TopLevelArgs;
use crate::cli::add::add_internal;
use crate::full_piece::FullPiece;
use crate::group::Group;
use crate::installation::Installation;
use crate::pieces::conf::Conf;
use crate::pieces::{BulkPieceEnum, PieceEnum};
use crate::utils::{confirm, press_enter};
use color_eyre::eyre::{Context, eyre};
use color_eyre::{Report, Result};
use ini_preserve::Ini;
use itertools::{EitherOrBoth, Itertools};
use log::info;
use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct Args {
    path: PathBuf,
}

#[expect(clippy::print_stdout)]
#[allow(clippy::needless_pass_by_value)]
pub fn watch_conf(top_level_args: TopLevelArgs, args: Args) -> Result<()> {
    let mut installation = Installation::get(&top_level_args)?;

    let get_values = || {
        if args.path.exists() {
            let ini = Ini::load(&args.path)
                .map_err(|e| eyre!(e))
                .wrap_err("Failed to parse ini file")?;
            Ok::<_, Report>(
                ini.sections()
                    .into_iter()
                    .flat_map(|section| {
                        ini.keys(section)
                            .into_iter()
                            .map(|(k, v)| ((section.to_string(), k.to_string()), v.to_string()))
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            Ok(Vec::new())
        }
    };

    let old_values = get_values()?;

    println!("Change settings and press enter when done");
    press_enter()?;

    let new_values = get_values()?;

    let changes: Vec<_> = old_values
        .into_iter()
        .merge_join_by(new_values, |(ka, _), (kb, _)| ka.cmp(kb))
        .map(|e| {
            Ok(match e {
                EitherOrBoth::Both((k, va), (_, vb)) => (k, Some(va), vb),
                // EitherOrBoth::Left((k, va)) => (k, Some(va), None),
                EitherOrBoth::Left(_) => return Err(eyre!("Removing keys is not supported")),
                EitherOrBoth::Right((k, vb)) => (k, None, vb),
            })
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|(_, old_value, new_value)| old_value.as_ref() != Some(new_value))
        .collect();

    if changes.is_empty() {
        info!("No changes found.");
        return Ok(());
    }

    let changes: Vec<_> = changes
        .into_iter()
        .filter_map(|((section, key), old_value, new_value)| {
            println!(
                "[{section}] {key}: {} -> {new_value}",
                old_value.as_ref().map_or("<missing>", |s| s)
            );
            match confirm("Do you want to add this change?") {
                Err(e) => Some(Err(eyre!(e))),
                Ok(false) => None,
                Ok(true) => Some(Ok(((section, key), old_value, new_value))),
            }
        })
        .collect::<Result<_>>()?;

    if changes.is_empty() {
        info!("All changes denied.");
        return Ok(());
    }

    add_internal(
        &mut installation,
        &top_level_args,
        changes
            .into_iter()
            .map(|((section, key), old_value, new_value)| {
                (
                    FullPiece::new(
                        PieceEnum::Bulk(BulkPieceEnum::Conf(Conf::new_with_original(
                            args.path.clone(),
                            section,
                            key,
                            new_value,
                            old_value,
                        ))),
                        None,
                        Group::All,
                    ),
                    true,
                )
            })
            .collect(),
    )?;

    Ok(())
}
