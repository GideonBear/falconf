# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
