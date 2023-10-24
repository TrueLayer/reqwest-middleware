# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

### [0.2.4] - 2023-09-21

### Added
- Added `fetch_mode_no_cors` method to `reqwest_middleware::RequestBuilder`

## [0.2.3] - 2023-08-07

### Added
- Added all `reqwest::Error` methods for `reqwest_middleware::Error`

## [0.2.2] - 2023-05-11

### Added
- `RequestBuilder::version` method to configure the HTTP version

## [0.2.1] - 2023-03-09

### Added
- Support for `wasm32-unknown-unknown`

## [0.2.0] - 2022-11-15

### Changed
- `RequestBuilder::try_clone` has a fixed function signature now

### Removed
- `RequestBuilder::send_with_extensions` - use `RequestBuilder::with_extensions` + `RequestBuilder::send` instead.

### Added
- Implementation of `Debug` trait for `RequestBuilder`.
- A new `RequestInitialiser` trait that can be added to `ClientWithMiddleware`
- A new `Extension` initialiser that adds extensions to each request
- Adds `with_extension` method functionality to `RequestBuilder` that can add extensions for the `send` method to use.

## [0.1.6] - 2022-04-21

Absolutely nothing changed

## [0.1.5] - 2022-02-21

### Added
- Added support for `opentelemetry` version `0.17`.

## [0.1.4] - 2022-01-24

### Changed
- Made `Debug` impl for `ClientWithExtensions` non-exhaustive.

## [0.1.3] - 2021-10-18

### Security
- remove time v0.1 dependency

### Fixed
- Handle the `hyper::Error(IncompleteMessage)` as a `Retryable::Transient`.

## [0.1.2] - 2021-09-28
### Changed
- Disabled default features on `reqwest`
- Replaced `truelayer-extensions` with `task-local-extensions`

## [0.1.1]
### Added
- New methods on `ClientWithExtensions` and `RequestBuilder` for sending requests with initial extensions.
