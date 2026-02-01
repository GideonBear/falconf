# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/GideonBear/falconf/compare/v0.2.21...v0.3.0) - 2026-02-01

### Added

- allow using '-' as a shortcut for the last piece in remove, edit, undo
- Add use FALCONF_PATH env variable

### Fixed

- *(deps)* update rust crate command-error to 0.8.0 ([#23](https://github.com/GideonBear/falconf/pull/23))
- *(deps)* update rust crate ron to 0.12.0 ([#24](https://github.com/GideonBear/falconf/pull/24))

### Other

- remove unused import
- *(deps)* update release-plz/action digest to e592230 ([#69](https://github.com/GideonBear/falconf/pull/69))
- *(deps)* lock file maintenance ([#71](https://github.com/GideonBear/falconf/pull/71))
- *(renovate)* disable rate-limiting ([#68](https://github.com/GideonBear/falconf/pull/68))
- *(renovate)* disable rate-limiting ([#67](https://github.com/GideonBear/falconf/pull/67))
- *(deps)* update dependency rust to v1.93.0 ([#65](https://github.com/GideonBear/falconf/pull/65))
- *(renovate)* migrate config
- *(renovate)* migrate config
- *(renovate)* add rust-toolchain updating
- *(deps)* update release-plz/action digest to a8c9cdc ([#64](https://github.com/GideonBear/falconf/pull/64))
- pin rust toolchain further ([#63](https://github.com/GideonBear/falconf/pull/63))
- drop testing for Windows and macOS
- [**breaking**] add and test MSRV
- *(deps)* update release-plz/action digest to 5ab144c ([#61](https://github.com/GideonBear/falconf/pull/61))
- *(deps)* lock file maintenance ([#62](https://github.com/GideonBear/falconf/pull/62))
- *(renovate)* use preset for lockfile config ([#60](https://github.com/GideonBear/falconf/pull/60))
- *(deps)* lock file maintenance ([#59](https://github.com/GideonBear/falconf/pull/59))
- *(deps)* update rust crate libc to v0.2.180 ([#58](https://github.com/GideonBear/falconf/pull/58))
- *(deps)* update rust crate indexmap to v2.13.0 ([#57](https://github.com/GideonBear/falconf/pull/57))
- *(deps)* update rust crate libc to v0.2.179 ([#54](https://github.com/GideonBear/falconf/pull/54))
- *(deps)* lock file maintenance ([#55](https://github.com/GideonBear/falconf/pull/55))
- *(renovate)* change semantic commit type to always be 'chore' ([#56](https://github.com/GideonBear/falconf/pull/56))
- remove deprecated authors field from Cargo.toml ([#53](https://github.com/GideonBear/falconf/pull/53))
- *(deps)* lock file maintenance ([#51](https://github.com/GideonBear/falconf/pull/51))
- *(deps)* lock file maintenance ([#50](https://github.com/GideonBear/falconf/pull/50))
- *(deps)* lock file maintenance ([#49](https://github.com/GideonBear/falconf/pull/49))
- *(lint_pr)* zizmor fixes ([#48](https://github.com/GideonBear/falconf/pull/48))
- *(renovate)* move Renovate config ([#47](https://github.com/GideonBear/falconf/pull/47))
- *(deps)* update rust crate shell-words to v1.1.1 ([#46](https://github.com/GideonBear/falconf/pull/46))
- *(deps)* lock file maintenance ([#45](https://github.com/GideonBear/falconf/pull/45))
- *(deps)* update rust crate ctor to v0.6.3 ([#42](https://github.com/GideonBear/falconf/pull/42))
- *(deps)* update release-plz/action digest to 487eb7b ([#43](https://github.com/GideonBear/falconf/pull/43))
- *(deps)* update rust crate git2 to v0.20.3 ([#44](https://github.com/GideonBear/falconf/pull/44))
- *(deps)* update rust crate libc to v0.2.178 ([#40](https://github.com/GideonBear/falconf/pull/40))
- *(deps)* update rust crate log to v0.4.29 ([#41](https://github.com/GideonBear/falconf/pull/41))
- *(deps)* update rust crate uuid to v1.19.0 ([#37](https://github.com/GideonBear/falconf/pull/37))
- *(deps)* update rust crate ctor to v0.6.2 ([#39](https://github.com/GideonBear/falconf/pull/39))
- *(deps)* update actions/checkout digest to 8e8c483 ([#38](https://github.com/GideonBear/falconf/pull/38))
- *(deps)* lock file maintenance ([#36](https://github.com/GideonBear/falconf/pull/36))
- *(deps)* update rust crate hostname to v0.4.2 ([#35](https://github.com/GideonBear/falconf/pull/35))
- fix typo in README.md
- fix test_file_dir not symlinking dir
- format imports in src/cli/mod.rs
- remove todos
- *(deps)* update swatinem/rust-cache digest to 779680d ([#34](https://github.com/GideonBear/falconf/pull/34))
- Update README.md
- *(deps)* update release-plz/action digest to 1efcf74 ([#33](https://github.com/GideonBear/falconf/pull/33))
- *(deps)* lock file maintenance ([#32](https://github.com/GideonBear/falconf/pull/32))
- *(deps)* update actions/checkout action to v6 ([#31](https://github.com/GideonBear/falconf/pull/31))
- *(deps)* update rust crate clap to v4.5.53 ([#30](https://github.com/GideonBear/falconf/pull/30))
- *(deps)* update actions/checkout digest to 93cb6ef ([#28](https://github.com/GideonBear/falconf/pull/28))
- *(deps)* update rust crate clap to v4.5.52 ([#29](https://github.com/GideonBear/falconf/pull/29))
- Update README.md
- *(lint_pr)* run on synchronize, and add zizmor ignore ([#27](https://github.com/GideonBear/falconf/pull/27))
- *(deps)* update rust crate libc to v0.2.177 ([#18](https://github.com/GideonBear/falconf/pull/18))
- *(deps)* update rust crate ctor to v0.6.1 ([#20](https://github.com/GideonBear/falconf/pull/20))
- *(deps)* update rust crate regex to v1.12.2 ([#22](https://github.com/GideonBear/falconf/pull/22))
- *(deps)* update actions/checkout action to v5 ([#25](https://github.com/GideonBear/falconf/pull/25))
- *(deps)* lock file maintenance ([#26](https://github.com/GideonBear/falconf/pull/26))
- *(deps)* pin dependencies ([#15](https://github.com/GideonBear/falconf/pull/15))
- *(deps)* update rust crate clap to v4.5.51 ([#16](https://github.com/GideonBear/falconf/pull/16))
- Add Renovate ([#14](https://github.com/GideonBear/falconf/pull/14))
- Enforce conventional commits in PR titles ([#13](https://github.com/GideonBear/falconf/pull/13))
- Document the fact that edit does not take a value

## [0.2.21](https://github.com/GideonBear/falconf/compare/v0.2.20...v0.2.21) - 2025-10-27

### Added

- *(list)* Add `undo_command` to `list` output
- *(edit)* Add `edit` subcommand
- *(logging)* Improve log messages for bulk pieces

### Fixed

- Fix help for some subcommands
- Fix add file help

### Other

- clippy
- Improve debug output in `list` test
- Add TODO's
- Add TODO
- Change 'shorthand' -> 'alias'
- *(todo)* Clarify and reprio failed push todo
- Make commit message meaningful

## [0.2.20](https://github.com/GideonBear/falconf/compare/v0.2.19...v0.2.20) - 2025-10-20

### Fixed

- undo/remove require at least one element

## [0.2.19](https://github.com/GideonBear/falconf/compare/v0.2.18...v0.2.19) - 2025-10-19

### Added

- add --undo for command pieces
- *(undo)* Allow passing multiple piece ids to undo

### Other

- Fix utils::prompt
- Add test_file_dir
- Improve test_sync
- Make sync and undo atomic
- Remove unnecessary to_path_buf call
- clippy
- Speed up testing by removing unecessary sleep for git daemon
- Update .gitignore
- Fix tests
- Add TODO
- Add falconf push to readme
- Clippy
- Add logging for piece execution/undoing
- Check for unsynced changes
- Remove dependency on built
- clippy

## [0.2.18](https://github.com/GideonBear/falconf/compare/v0.2.17...v0.2.18) - 2025-09-26

### Other

- Simplify and categorize TODOs
- Improve an `#[expect]`
- Fix file
- Remove unused argument
- Canonicalize paths
- Remove `--dry-run`
- Guard against data file changes during dry-run
- Add push command
- Add sanity check
- cargo update
- Fix wrong method call
- Fix empty commits
- Fix clap error
- Fix machines being randomly reordered in data file
- Clean up merge analysis
- Remove hacky `git add .`
- TODOs

## [0.2.17](https://github.com/GideonBear/falconf/compare/v0.2.16...v0.2.17) - 2025-09-21

### Other

- Fix release_assets workflow

## [0.2.16](https://github.com/GideonBear/falconf/compare/v0.2.15...v0.2.16) - 2025-09-21

### Other

- Fix release_assets workflow

## [0.2.15](https://github.com/GideonBear/falconf/compare/v0.2.14...v0.2.15) - 2025-09-21

### Other

- Fix release_assets workflow
- Fix release_assets workflow

## [0.2.14](https://github.com/GideonBear/falconf/compare/v0.2.13...v0.2.14) - 2025-09-21

### Other

- Remove unnecessary path

## [0.2.13](https://github.com/GideonBear/falconf/compare/v0.2.12...v0.2.13) - 2025-09-21

### Other

- Fix release-plz workflow

## [0.2.12](https://github.com/GideonBear/falconf/compare/v0.2.11...v0.2.12) - 2025-09-21

### Other

- Update comment

## [0.2.11](https://github.com/GideonBear/falconf/compare/v0.2.10...v0.2.11) - 2025-09-21

### Other

- Fix release_assets workflow
- Fix release_assets workflow
- Switch to release-plz
- Update README.md
- Fix clippy
- Fix piece ordering in sync
- Remove todo
- Remove dead code
- Deduplicate execute_bulk and undo_bulk
- Improve `impl Display for PieceEnum`
- Split dry_run and test_run, split Piece into BulkPiece and NonBulkPiece, steps towards atomic execution
- Add TODO
- Move *Args to respective modules
- Update ctor
- Remove TODO
- Remove FullPiece.{undo,one_time} in favour of undone_on and one_time_todo_on being Some/None
- Update README.md
- Update README.md
- Fix short args
- Add TODOs
- Add short args for some piece types
- Update README.md
- :not_done_here -> done_here
- Add remove command
- Don't strikethrough unused suffix in list
- Fix strikethrough
- Fix tests
- Improve list colors
- Make unused suffic italic
- Add comment to release workflow
