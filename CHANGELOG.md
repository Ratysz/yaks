# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/Ratysz/yaks/compare/0.1.0..HEAD)
### Added
- `resources-interop` feature: when enabled, allows `Executor::run()` to also
accept `Resources` struct from the `resources` crate in place of resources argument.
- `SystemContext::reserve_entity()`, `::contains()`, `::archetypes()`,
and `::archetype_generation()`, mirroring similar methods of `hecs::World`.
- CI badge.
### Changed
- `Executor::run()` now uses `rayon::scope_fifo()`.
- Minor doc tweaks.
- Fixed changelog dates.
- Internal refactors.
### Removed
- `test` feature.

## [0.1.0](https://github.com/Ratysz/yaks/compare/0.0.0-aplha1..0.1.0) - 2020-06-06
### Changed
- Full rewrite.

## [0.0.0-aplha1](https://github.com/Ratysz/yaks/releases/tag/0.0.0-aplha1) - 2020-01-14
### Added
- Initial release.