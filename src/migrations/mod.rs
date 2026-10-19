mod migration_2;
mod migration_3;
mod migration_4;
mod migration_5;

use crate::data::{from_file, to_file};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use itertools::Itertools;
use log::info;
use seq_macro::seq;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::Path;

pub(crate) const VERSION: u32 = 5;

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
    seq!(N in 2..=5 {
        match migration {
            #( N => migrate_generic::<migration_~N::OldData, migration_~N::NewData>(path), )*
            #[expect(clippy::missing_panics_doc)]
            #[expect(clippy::panic)]
            _ => panic!("All migrations up to VERSION should be present in the match"),
        }
    })
}

fn migrate_generic<OldData, NewData>(path: &Path) -> Result<()>
where
    OldData: DeserializeOwned,
    NewData: Serialize,
    OldData: Into<NewData>,
{
    to_file::<NewData>(&from_file::<OldData>(path)?.into(), path)
}

// Necessary because of https://github.com/ron-rs/ron/issues/631
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct RawValue(Box<ron::value::RawValue>);

impl<'de> Deserialize<'de> for RawValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = Box::<ron::value::RawValue>::deserialize(d)?;
        Ok(Self(
            ron::value::RawValue::from_ron(raw.get_ron().trim())
                .map_err(serde::de::Error::custom)?
                .to_owned(),
        ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::from_reader;
    use crate::data::to_writer;
    use include_optional::include_bytes_optional;
    use with_builtin_macros::with_eager_expansions;

    macro_rules! test_migration {
        ($from:literal, $migration:literal) => {
            with_eager_expansions! {
                #[test]
                fn #{ concat_idents!(test_migration_, $migration) }() -> Result<()> {
                    let old_bytes = include_bytes!(concat!("test_data/version_", $from, ".ron"));
                    // include_bytes_optional! is not a built-in macro so this needs with_eager_expansions
                    // stringify! necessary because of https://github.com/danielhenrymantilla/with_builtin_macros.rs/issues/3
                    let new_expected: Option<&[u8]> = include_bytes_optional!(#{ concat!("test_data/version_", stringify!($migration), ".ron") });
                    let old: #{ concat_idents!(migration_, $migration) }::OldData = from_reader(old_bytes.as_slice())?;
                    let new: #{ concat_idents!(migration_, $migration) }::NewData = old.into();
                    if let Some(new_expected) = new_expected {
                        let mut writer = vec![];
                        to_writer(&new, &mut writer)?;
                        assert_eq!(new_expected, writer);
                    } else {
                        to_file(
                            &new,
                            &Path::new(file!())
                                .parent()
                                .unwrap()
                                .join(concat!("test_data/suggested_version_", $migration, ".ron")),
                        )?;
                        // TODO: add possibility to break chain instead of restarting from version 1
                        panic!(concat!(
                            "Missing test data. Wrote `suggested_version_", $migration, ".ron`.\n\
                            Run `diff src/migrations/test_data/{{version_", $from,",suggested_version_", $migration, "}}.ron` to see the migration diff, and \
                            run `mv src/migrations/test_data/{{suggested_,}}version_", $migration, ".ron` if it is correct.\n\
                            If it does not test all relevant aspects of the migration, edit `version_1.ron` and regenerate all following versions."
                        ));
                    }
                    Ok(())
                }
            }
        };
    }

    test_migration!(1, 2);
    test_migration!(2, 3);
    test_migration!(3, 4);
    test_migration!(4, 5);
    const CURRENT_BYTES: &[u8] = include_bytes!("test_data/version_5.ron");

    #[test]
    fn test_current_parses() -> Result<()> {
        from_reader::<crate::data::Data>(CURRENT_BYTES)?;
        Ok(())
    }

    #[test]
    fn test_current_remains_unchanged() -> Result<()> {
        let data = from_reader::<crate::data::Data>(CURRENT_BYTES)?;
        let mut writer = vec![];
        to_writer(&data, &mut writer)?;
        assert_eq!(CURRENT_BYTES, writer);
        Ok(())
    }
}
