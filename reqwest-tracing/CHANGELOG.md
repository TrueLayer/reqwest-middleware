# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3] - 2022-06-23
### Fixed
- Fix how we set the OpenTelemetry span status, based on the HTTP response status.

# [0.2.2] - 2022-04-21
### Fixed
- Opentelemetry context is now propagated when the request span is disabled.

## [0.2.1] - 2022-02-21
### Changed
- Updated `reqwest-middleware` to `0.1.5`

## [0.2.0] - 2021-11-30
### Breaking
- Update to `tracing-subscriber` `0.3.x` when `opentelemetry_0_16` is active.

## [0.1.3] - 2021-09-28
### Changed
- Disabled default features on `reqwest`
- Replaced `truelayer-extensions` with `task-local-extensions`
- Updated `reqwest-middleware` to `0.1.2`

## [0.1.2] - 2021-09-15
### Changed
- Updated `reqwest-middleware` dependency to `0.1.1`.

## [0.1.1] - 2021-08-30
### Added
- Support for opentelemtry `0.15` and `0.16`.
