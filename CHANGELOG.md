# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release with core functionality

## [0.1.0] - YYYY-MM-DD

### Added
- Core iterator operations: filter, map, take, skip, etc.
- CSV/TSV/JSON input parsing via CLI flags
- Multiple output formats (debug, json, jsonl, csv)
- Caching of compiled binaries for fast re-execution
- Comprehensive error messages with suggestions
- Join operations (inner, left)
- Grouping operations (chunk, window, group_by)
- File input support

### Infrastructure
- CI/CD with GitHub Actions
- Multi-platform releases (Linux, macOS, Windows)
- Embedded toolchain support for self-contained binaries

[Unreleased]: https://github.com/olirice/lob/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/olirice/lob/releases/tag/v0.1.0
