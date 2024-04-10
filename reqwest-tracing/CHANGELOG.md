# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2024-04-10

### Breaking changes
- Upgraded `reqwest-middleware` to `0.3.0`.
- Removed support for `opentelemetry` 0.13 to 0.19

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
