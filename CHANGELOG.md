# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
