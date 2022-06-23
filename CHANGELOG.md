# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to the adaptation of [Semantic Versioning](https://semver.org/spec/v2.0.0.html) utilized by [Cargo](https://doc.rust-lang.org/cargo/reference/semver.html).

## [0.0.2] - 2022-xx-xx
### Changed
- Replace initialization functions with a compile-time friendly API that eliminates any potential virtual calls.
- Documentation and examples.

### Added
- `EventHandler` interface.
- `EventContext` type which implements `LambdaContext`.

### Removed
- `RefLambdaContext` type which previously implemented `LambdaContext`.

## [0.0.1] - 2022-05-22
### Added

- Core interface traits and default implementations.
- Runtime interface and a default implementation.
- Macros for easy creation of runtimes.
- Documentation.
- `ureq` based HTTP backend.
- Echo server example.