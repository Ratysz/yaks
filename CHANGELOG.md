# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--## [Unreleased](https://github.com/Ratysz/yaks/compare/0.0.0..HEAD)-->

## [Unreleased](https://github.com/Ratysz/yaks/compare/0.0.0-aplha1..HEAD)
### Added
- `ComponentBundle` and `EntityBuilder` are now in public API.
- `macros` feature: enables `hecs/macros`.
- `parallel` feature: enables `Executor::run_parallel()` (not implemented), exposes
`Threadpool` trait, an argument implementing which is required by `run_parallel()`.
- `impl_scoped_threadpool` feature: re-exports `scoped_threadpool` crate and implements
`Threadpool` for `scoped_threadpool::Pool`.
### Changed
- Documentation pass (`error`, partially `World`).
- `Executor` methods now take an `Into<SystemInsertionArguments>`, implemented on
`System` and tuples bundling it with either/both a handle and a vector of dependencies.

## [0.0.0-aplha1](https://github.com/Ratysz/yaks/releases/tag/0.0.0-aplha1)  - 2019-1-14
### Added
- Initial release.