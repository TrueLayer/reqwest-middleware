# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Added support fro OpenTelemetry `0.31` ([#250](https://github.com/TrueLayer/reqwest-middleware/pull/250))


## [0.6.0] - 2026-01-07

### Changed

- Updated `reqwest` to `0.13`

## [0.5.8] - 2025-06-16

### Added
- Added support for OpenTelemetry `0.30` ([#236](https://github.com/TrueLayer/reqwest-middleware/pull/236))

## [0.5.7] - 2025-04-08

### Added
- Added support for OpenTelemetry `0.29` ([#228](https://github.com/TrueLayer/reqwest-middleware/pull/228))

## [0.5.6] - 2025-02-24

### Added
- Added support for OpenTelemetry `0.28` ([#215](https://github.com/TrueLayer/reqwest-middleware/pull/215))

## [0.5.5] - 2024-12-02

### Added
- Added support for OpenTelemetry `0.27` ([#201](https://github.com/TrueLayer/reqwest-middleware/pull/201))

## [0.5.4] - 2024-11-08

### Added
- Added support for OpenTelemetry `0.25` ([#188](https://github.com/TrueLayer/reqwest-middleware/pull/188))
- Added support for OpenTelemetry `0.26` ([#188](https://github.com/TrueLayer/reqwest-middleware/pull/188))
- Added support reqwest-middleware `0.4` next to `0.3`

### Changed
- Restore adding `http.url` attribute when using `SpanBackendWithUrl` middleware with the `deprecated_attributes` feature enabled

## [0.5.3] - 2024-07-15

### Added
- Added support for OpenTelemetry `0.24` ([#171](https://github.com/TrueLayer/reqwest-middleware/pull/171))

### Fixed
- Fixed, `deprecated_attributes` feature, failing to compile ([#172](https://github.com/TrueLayer/reqwest-middleware/pull/172))

## [0.5.2] - 2024-07-15

### Added
- Added feature flag, `deprecated_attributes`, for emitting [deprecated opentelemetry HTTP attributes](https://opentelemetry.io/docs/specs/semconv/http/migration-guide/) alongside the stable ones used by default

## [0.5.1] - 2024-06-28

### Added
- Added support for `opentelemetry` version `0.23`.

## [0.5.0] - 2024-04-10

### Breaking changes
- Upgraded `reqwest-middleware` to `0.3.0`.
- Removed support for `opentelemetry` 0.13 to 0.19
- The keys emitted by the crate now match the stable Semantic Conventions for HTTP Spans.

### Changed
- The keys emitted by the crate now match the stable Semantic Conventions for HTTP Spans.
- Opentelemetry features are now additive.

## [0.4.8] - 2024-03-11

### Added
- Add support for opentelemetry 0.22

## [0.4.6] - 2023-08-23

### Added
- Add support for opentelemetry 0.20

## [0.4.5] - 2023-06-20

### Added
- A new extension `DisableOtelPropagation` which stops opentelemetry contexts propagating
- Support for opentelemetry 0.19

## [0.4.4] - 2023-05-15

### Added
- A new `default_span_name` method for use in custom span backends.

## [0.4.3] - 2023-05-15

### Fixed
- Fix span and http status codes

## [0.4.2] - 2023-05-12

### Added
- `OtelPathNames` extension to provide known parameterized paths that will be used in span names

### Changed
- `DefaultSpanBackend` and `SpanBackendWithUrl` default span name to HTTP method name instead of `reqwest-http-client`

## [0.4.1] - 2023-03-09

### Added

- Support for `wasm32-unknown-unknown` target

## [0.4.0] - 2022-11-15

### Changed
- Updated `reqwest-middleware` to `0.2.0`
- Before, `root_span!`/`DefaultSpanBacked` would name your spans `{METHOD} {PATH}`. Since this can be quite
  high cardinality, this was changed and now the macro requires an explicit otel name.
  `DefaultSpanBacked`/`SpanBackendWithUrl` will default to `reqwest-http-client` but this can be configured
  using the `OtelName` Request Initialiser.

### Added
- `SpanBackendWithUrl` for capturing `http.url` in traces
- `OtelName` Request Initialiser Extension for configuring

## [0.3.1] - 2022-09-21
- Added support for `opentelemetry` version `0.18`.

## [0.3.0] - 2022-06-10
### Breaking
- Created `ReqwestOtelSpanBackend` trait with `reqwest_otel_span` macro to provide extendable default request otel fields

## [0.2.3] - 2022-06-23
### Fixed
- Fix how we set the OpenTelemetry span status, based on the HTTP response status.

## [0.2.2] - 2022-04-21
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
