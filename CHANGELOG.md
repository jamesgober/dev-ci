# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0] - 2026-05-11

### Added

- Initial crate skeleton.
- `Target` enum for supported CI platforms. `GitHubActions` is the first.
- `Generator` builder with `target`, `with_clippy`, `with_fmt`, `with_msrv` configuration methods.
- `Generator::generate` produces workflow YAML.
- GitHub Actions workflow uses `actions/checkout@v5` (Node 24-compatible).
- Smoke tests covering: default generation, optional Clippy job, optional rustfmt job, MSRV job with pinned toolchain.

### Note

This is the foundation release. The full feature set — multi-target
support (GitLab, Buildkite, CircleCI), the GitHub Action runtime, PR
annotations, SARIF upload, and the `dev-tools` integration that auto-
detects enabled producers — lands across the `0.9.x` line.

[Unreleased]: https://github.com/jamesgober/dev-ci/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/jamesgober/dev-ci/releases/tag/v0.9.0
