//! Dynamic shell completion candidates. These read the local data file directly
//! (without pulling), so completion stays fast and works offline. That duplicates
//! some things out of necessity.

use crate::cli::DEFAULT_PATH;
use crate::data::{self, Data};
use crate::installation::Installation;
use crate::repo::DATA_PATH;
use crate::utils::print_id_raw;
use clap_complete::CompletionCandidate;
use expanduser::expanduser;
use std::collections::BTreeSet;
use std::env;

// TODO(low): this ignores `--path` on the command line, because completers
//  don't get access to the other arguments.
fn read_data() -> Option<Data> {
    let root = env::var("FALCONF_PATH").unwrap_or_else(|_| DEFAULT_PATH.to_owned());
    let root = expanduser(root).ok()?;
    data::from_file(&Installation::get_repository_path(&root).join(DATA_PATH)).ok()
}

pub fn pieces() -> Vec<CompletionCandidate> {
    let Some(data) = read_data() else {
        return vec![];
    };
    data.pieces()
        .iter()
        .map(|(&id, piece)| {
            CompletionCandidate::new(print_id_raw(id)).help(Some(piece.piece.to_string().into()))
        })
        .collect()
}

pub fn machines() -> Vec<CompletionCandidate> {
    let Some(data) = read_data() else {
        return vec![];
    };
    std::iter::once(CompletionCandidate::new("this").help(Some("The current machine".into())))
        .chain(
            data.machines()
                .values()
                .map(|machine_data| CompletionCandidate::new(&machine_data.name)),
        )
        .collect()
}

fn custom_groups(data: &Data) -> BTreeSet<&str> {
    data.machines()
        .values()
        .flat_map(|machine_data| machine_data.groups.iter().map(String::as_str))
        .collect()
}

/// Groups a machine can be added to or removed from
pub fn machine_groups() -> Vec<CompletionCandidate> {
    let Some(data) = read_data() else {
        return vec![];
    };
    custom_groups(&data)
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

/// Groups/machines a piece can run on
pub fn piece_groups() -> Vec<CompletionCandidate> {
    let Some(data) = read_data() else {
        return vec![];
    };
    ["all", "none"]
        .into_iter()
        .chain(
            data.machines()
                .values()
                .map(|machine_data| machine_data.name.as_str()),
        )
        .chain(custom_groups(&data))
        .map(CompletionCandidate::new)
        .collect()
}
