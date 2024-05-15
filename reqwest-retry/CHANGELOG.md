# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added `regex` feature to
- Added `with_retry_log_level` to `RetryTransientMiddleware`

## [0.5.0] - 2024-04-10

### Breaking changes
- Upgraded `reqwest-middleware` to `0.3.0`.

## [0.3.0] - 2023-09-07
### Changed
- `retry-policies` upgraded to 0.2.0

## [0.2.3] - 2023-08-30
### Added
- `RetryableStrategy` which allows for custom retry decisions based on the response that a request got

## [0.2.1] - 2022-12-01

### Changed
- Classify `io::Error`s and `hyper::Error(Canceled)` as transient

## [0.2.0] - 2022-11-15
### Changed
- Updated `reqwest-middleware` to `0.2.0`

## [0.1.4] - 2022-02-21
### Changed
- Updated `reqwest-middleware` to `0.1.5`

## [0.1.3] - 2022-01-24
### Changed
- Updated `reqwest-middleware` to `0.1.4`

## [0.1.2] - 2021-09-28
### Added
- Re-export `RetryPolicy` from the crate root.
### Changed
- Disabled default features on `reqwest`
- Replaced `truelayer-extensions` with `task-local-extensions`
- Updated `reqwest-middleware` to `0.1.2`

## [0.1.1] - 2021-09-15
### Changed
- Updated `reqwest-middleware` dependency to `0.1.1`.
