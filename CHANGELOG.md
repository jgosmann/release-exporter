# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.1] - 2024-11-23

### Changed

* Updated dependencies.


## [0.5.0] - 2023-11-12

### Changed

* Depend on OpenSSL 3.
* Updated dependencies.


## [0.4.0] - 2023-02-02

### Added

* Added `version_regex` and `version_fmt` settings to `prometheus` provider.

### Changed

* Renamed `tag_name_regex` to `version_regex` for the `latest_github_release`
  provider.


## [0.3.1] - 2023-01-01

### Fixed

* Expiry of cache items.

## [0.3.0] - 2023-01-01

### Added

* Ability to cache fetched release information.


## [0.2.0] - 2022-12-31

* Add `release_exporter_build_info` metric.
* Build on ubuntu-20.04 instead of ubuntu-latest avoid dependency on OpenSSL 3.

## [0.1.0] - 2022-12-31

Initial release.

[Unreleased]: https://github.com/jgosmann/release-exporter/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/jgosmann/release-exporter/releases/tag/v0.5.1
[0.5.0]: https://github.com/jgosmann/release-exporter/releases/tag/v0.5.0
[0.4.0]: https://github.com/jgosmann/release-exporter/releases/tag/v0.4.0
[0.3.1]: https://github.com/jgosmann/release-exporter/releases/tag/v0.3.1
[0.3.0]: https://github.com/jgosmann/release-exporter/releases/tag/v0.3.0
[0.2.0]: https://github.com/jgosmann/release-exporter/releases/tag/v0.2.0
[0.1.0]: https://github.com/jgosmann/release-exporter/releases/tag/v0.1.0

