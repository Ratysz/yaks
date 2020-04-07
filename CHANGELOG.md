# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/Ratysz/yaks/compare/0.0.0-aplha1..HEAD)
### Removed
- `World`: systems and executors now take separate references to `hecs::World`,
`resources::Resources`, and `ModQueuePool`.
- `error` module, errors are now in crate root.
### Added
- `SystemContext`, bundles up state references in system closures.
- `ModQueuePool`, `Runnable`, `ArchetypeAccess`, and `SystemBorrows` are now in public API.
- `ExecutorBuilder`, splits off builder-like methods from the `Executor`.
- `parallel` feature: enables `System::run_with_scope()`, `Executor::run_with_scope()`, and
`Executor::run_parallel()`, exposes `Threadpool` and `Scope` - a scoped threadpool
implementation that can be used with those.
- `FetchResources`, a helper trait implemented on `resources::Resources` - allows getting
multiple resources via an API similar to `hecs` queries.
- `FetchComponents`, a helper trait implemented on `hecs::World` - allows getting
multiple components from an entity via an API similar to `hecs` queries.
### Changed
- Documentation pass (`NoSuchSystem`).
- Refactored `Executor` system insertion methods, split off builder-like methods
into `ExecutorBuilder`.
- `Executor` now solves system dependencies and sorts systems as they are inserted.

## [0.0.0-aplha1](https://github.com/Ratysz/yaks/releases/tag/0.0.0-aplha1)  - 2019-1-14
### Added
- Initial release.