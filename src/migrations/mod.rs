mod migration_2;
mod migration_3;

use crate::data::{from_file, to_file};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use itertools::Itertools;
use log::info;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::Path;

pub(crate) const VERSION: u32 = 3;

#[derive(Debug, Clone, Deserialize)]
struct Data {
    version: u32,
}

pub(crate) fn run_migrations(path: &Path) -> Result<Option<String>> {
    let Data { version } = from_file(path)?;
    if version == VERSION {
        return Ok(None);
    } else if version > VERSION {
        return Err(eyre!(
            "Your data file is newer than this falconf version supports. Please update falconf."
        ));
    }

    let mut ran = vec![];

    for migration in (version + 1)..=VERSION {
        info!("Running migration to data file version {migration}...");
        migrate(migration, path)?;
        ran.push(migration);
    }

    Ok(Some(
        ran.into_iter().map(|s| format!("migration {s}")).join(", "),
    ))
}

fn migrate(migration: u32, path: &Path) -> Result<()> {
    match migration {
        2 => {
            to_file::<migration_2::NewData>(&from_file::<migration_2::OldData>(path)?.into(), path)
        }
        3 => {
            to_file::<migration_3::NewData>(&from_file::<migration_3::OldData>(path)?.into(), path)
        }
        #[expect(clippy::missing_panics_doc)]
        #[expect(clippy::panic)]
        _ => panic!("All migrations up to VERSION should be present in the match"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LitU32<const N: u32>;

impl<const N: u32> Serialize for LitU32<N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u32(N)
    }
}

impl<'de, const N: u32> Deserialize<'de> for LitU32<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = u32::deserialize(d)?;
        if v == N {
            Ok(LitU32)
        } else {
            Err(serde::de::Error::custom(format!("expected {N}, got {v}")))
        }
    }
}
