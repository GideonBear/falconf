use crate::full_piece::FullPiece;
use crate::group::Group;
use crate::machine::{Machine, MachineData};
use crate::migrations;
use crate::utils::print_id;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Group names with a special meaning, see [`Group`]
const RESERVED_GROUPS: [&str; 2] = ["all", "none"];

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    version: u32,
    pieces: IndexMap<u32, FullPiece>,
    machines: IndexMap<Machine, MachineData>,
}

impl Data {
    pub fn init_new() -> Self {
        Self {
            version: migrations::VERSION,
            pieces: IndexMap::new(),
            machines: IndexMap::new(),
        }
    }

    pub const fn pieces(&self) -> &IndexMap<u32, FullPiece> {
        &self.pieces
    }

    pub const fn pieces_mut(&mut self) -> &mut IndexMap<u32, FullPiece> {
        &mut self.pieces
    }

    pub const fn machines(&self) -> &IndexMap<Machine, MachineData> {
        &self.machines
    }

    pub const fn machines_mut(&mut self) -> &mut IndexMap<Machine, MachineData> {
        &mut self.machines
    }

    pub const fn as_mut_parts(
        &mut self,
    ) -> (
        &mut IndexMap<u32, FullPiece>,
        &mut IndexMap<Machine, MachineData>,
    ) {
        (&mut self.pieces, &mut self.machines)
    }

    /// Check invariants that the types don't enforce. These can be broken by a git merge of the
    /// data file, so this is also checked on every load.
    pub fn validate(&self) -> Result<()> {
        let mut names = HashSet::new();
        for machine_data in self.machines.values() {
            let name = &machine_data.name;
            if RESERVED_GROUPS.contains(&name.as_str()) {
                return Err(eyre!(
                    "'{name}' is reserved and can't be used as a machine name"
                ));
            }
            if !names.insert(name.as_str()) {
                return Err(eyre!(
                    "Machine name '{name}' is already used by another machine"
                ));
            }
        }
        let piece_groups = self
            .pieces
            .iter()
            .filter_map(|(&id, piece)| match &piece.group {
                Group::Custom(group) => Some(Ok(group)),
                Group::Machine(machine) if !self.machines.contains_key(machine) => {
                    Some(Err(eyre!(
                        "Piece {} is grouped on machine {}, which doesn't exist",
                        print_id(id),
                        machine.0
                    )))
                }
                Group::All | Group::None | Group::Machine(_) => None,
            });
        let machine_groups = self.machines.values().flat_map(|m| &m.groups).map(Ok);
        for group in machine_groups.chain(piece_groups) {
            let group = group?;
            if RESERVED_GROUPS.contains(&group.as_str()) {
                return Err(eyre!("'{group}' is reserved and can't be used as a group"));
            }
            if names.contains(group.as_str()) {
                return Err(eyre!(
                    "'{group}' can't be both a machine name and a group: a machine name is already implicitly a group containing that machine"
                ));
            }
        }
        Ok(())
    }
}

pub fn from_reader<T: DeserializeOwned>(reader: impl Read) -> Result<T> {
    Ok(ron::de::from_reader(reader)?)
}

pub fn from_file<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    from_reader(reader)
}

pub fn to_writer<T: Serialize>(data: &T, writer: impl Write) -> Result<()> {
    Ok(ron::Options::default().to_io_writer_pretty(
        writer,
        data,
        ron::ser::PrettyConfig::default(),
    )?)
}

pub fn to_file<T: Serialize>(data: &T, path: &Path) -> Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    to_writer(data, writer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pieces::falconf_init::FalconfInit;
    use crate::pieces::{NonBulkPieceEnum, PieceEnum};
    use std::collections::BTreeSet;

    fn with_piece(mut data: Data, group: Group) -> Data {
        let piece = PieceEnum::NonBulk(NonBulkPieceEnum::FalconfInit(FalconfInit::new(
            "remote".to_string(),
        )));
        data.pieces
            .insert(1, FullPiece::new(piece, None, group, BTreeSet::new()));
        data
    }

    fn data_with(machines: &[(&str, &[&str])]) -> Data {
        let mut data = Data::init_new();
        for (name, groups) in machines {
            data.machines.insert(
                Machine::new(),
                MachineData {
                    name: (*name).to_string(),
                    groups: groups
                        .iter()
                        .map(|g| (*g).to_string())
                        .collect::<BTreeSet<_>>(),
                },
            );
        }
        data
    }

    #[test]
    fn test_validate() {
        assert!(
            data_with(&[("a", &["g"]), ("b", &["g"])])
                .validate()
                .is_ok()
        );
        assert!(data_with(&[("a", &[]), ("a", &[])]).validate().is_err());
        assert!(data_with(&[("a", &[]), ("b", &["a"])]).validate().is_err());
        assert!(data_with(&[("all", &[])]).validate().is_err());
        assert!(data_with(&[("a", &["none"])]).validate().is_err());

        let data = data_with(&[("a", &[])]);
        let a = *data.machines.keys().next().unwrap();
        assert!(
            with_piece(data.clone(), Group::Machine(a))
                .validate()
                .is_ok()
        );
        assert!(
            with_piece(data.clone(), Group::Custom("g".to_string()))
                .validate()
                .is_ok()
        );
        assert!(
            with_piece(data.clone(), Group::Machine(Machine::new()))
                .validate()
                .is_err()
        );
        assert!(
            with_piece(data.clone(), Group::Custom("a".to_string()))
                .validate()
                .is_err()
        );
        assert!(
            with_piece(data, Group::Custom("all".to_string()))
                .validate()
                .is_err()
        );
    }
}
