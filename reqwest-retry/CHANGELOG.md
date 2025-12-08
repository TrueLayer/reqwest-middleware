# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added `with_retry_log_level` to `RetryTransientMiddleware` with custom `RetryableStrategy` in reqwest-retry

## [0.8.0] - 2025-11-26

### Breaking Changes

- Updated `retry-policies` (re-exported as `reqwest_retry::policies`) to 0.5.

### Changed

- Updated `thiserror` to `2.0`

## [0.7.0] - 2024-11-08

### Breaking changes
- Errors are now reported as `RetryError` that adds the number of retries to the error chain if there were any. This changes the returned error types.

### Added
- Added support reqwest-middleware `0.4` next to `0.3`

## [0.6.1] - 2024-08-08

### Added
- Removed dependency on `chrono` ([#170](https://github.com/TrueLayer/reqwest-middleware/pull/170))

## [0.6.0] - 2024-06-28

### Added
- Added `with_retry_log_level` to `RetryTransientMiddleware`

### Changed
- Upgraded `retry-policies` to `0.4.0`.

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
