# Changelog

All notable changes to LazyCelery will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial implementation of LazyCelery
- Real-time worker monitoring with status and utilization tracking
- Task management with filtering, search, and scrollable views
- Queue visualization with message counts and consumer status
- Keyboard-driven interface with Tab navigation
- Async Redis broker support with trait-based architecture
- Configuration file support (TOML)
- Comprehensive test suite with 31 passing tests
- CI/CD pipelines with GitHub Actions
- Automatic changelog generation with git-cliff
- Docker support for containerized deployment
- Security auditing with cargo-audit and cargo-deny

### Fixed
- Task list scrolling with proper viewport management
- Added scroll position indicators
- Fixed all compilation warnings

## [0.1.0] - TBD

Initial release

[Unreleased]: https://github.com/fguedes90/lazycelery/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/fguedes90/lazycelery/releases/tag/v0.1.0