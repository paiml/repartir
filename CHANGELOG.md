# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive test suite with 92 tests achieving 88.58% code coverage
- pmat quality enforcement with pre-commit hooks (TDG score: 94.6/100, Grade A)
- Mock TCP server for remote executor testing
- TLS configuration validation and error handling tests
- CPU executor edge case tests (large output, empty output, signal termination)
- Coverage report files added to .gitignore

### Changed
- Improved remote executor test coverage from 28.92% to 80.60%
- Improved TLS executor test coverage from 40.46% to 70.70%
- Improved CPU executor test coverage from 93.14% to 94.73%
- Overall project coverage improved from 64.97% to 88.58%

### Fixed
- Pre-commit hook now correctly analyzes only library code (excludes src/bin/)
- SATD check updated to match current pmat output format

## [0.1.0] - 2024-01-01

### Added
- Initial release with v1.1 features complete
- CPU executor with async process execution
- GPU executor with wgpu compute support (skeleton)
- Remote executor with TCP-based task distribution
- TLS support for secure remote connections
- PubSub and PushPull messaging patterns
- Work-stealing task scheduler
- Comprehensive error handling with RepartirError
- Iron Lotus Framework principles (Jidoka, Kaizen, Genchi Genbutsu)
- Certeza testing methodology (Tier 1: sub-second)

### Dependencies
- tokio 1.35+ for async runtime
- wgpu 23.0+ for GPU compute
- rustls 0.23+ for TLS (with aws-lc-rs)
- serde 1.0+ for serialization
- uuid 1.6+ for stable task IDs

[Unreleased]: https://github.com/paiml/repartir/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/paiml/repartir/releases/tag/v0.1.0
