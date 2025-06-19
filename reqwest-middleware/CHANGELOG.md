# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added `ClientBuilder::with_final_init` and `ClientBuilder::with_arc_final_init`

## [0.4.2] - 2025-04-08

### Added
- Deprecated `fetch_mode_no_cors` as it's been deprecated in reqwest.

## [0.4.1] - 2025-02-24

- Fixed wasm32 by disabling incompatible parts. On that target, `ClientWithMiddleware` is no longer
  a Tower service and has no `ClientWithMiddleware::timeout` function.

### Changed
- Updated `wasm-timer` to `wasmtimer`

## [0.4.0] - 2024-11-08

### Breaking Changes
- `request_middleware::Error` is now a transparent error enum and doesn't add its own context anymore.

## [0.3.3] - 2024-07-08

### Added
- Implemented `Default` on `ClientWithMiddleware` ([#179](https://github.com/TrueLayer/reqwest-middleware/pull/179))

## [0.3.2] - 2024-06-28

### Added
- Added re-export of `reqwest`.
- `http2`, `rustls-tls`, and `charset` features, which simply enable those features in `reqwest`.

## [0.3.1]

### Fixed
- Included license files in crates
- Fix logging of User-Agent header in reqwest-tracing

### Added
- Added `with_retry_log_level` to `RetryTransientMiddleware` in reqwest-retry
- Added `ClientBuilder::from_client`

## [0.3.0] - 2024-04-10

### Breaking changes
- Upgraded `reqwest` to `0.12.0`
  * Removed default-features `json` and `multipart` from `reqwest` dependency
  * Added `json` and `multipart` features to `reqwest-middleware`
- Upgraded `matchit` to `0.8.0`
  * You may need to update some matches that look like `/a/:some_var` to `/a/{some_var}`
- Removed `task_local_extensions` in favour of `http::Extensions`
  * All extensions must be `Clone` now.

### Changed
- `RequestBuilder::try_clone` now clones the extensions.

### Added
- Implemented `Service` for `ClientWithMiddleware` to have more feature parity with `reqwest`.
- Added more methods like `build_split` to have more feature parity with `reqwest.`
- Added more documentation

### [0.2.5] - 2024-03-15

### Changed
- Updated minimum version of `reqwest` to `0.11.10`. url_mut, with_url, without_url functions are added after `0.11.10`.

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
