# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0] - 2026-05-12

Foundation release. Replaces the `0.1.0` name-claim with the full
generator + CLI surface for GitHub Actions output.

### Added

- `Generator` builder gains the full standard surface: `workflow_name`, `branches(iter)`, `matrix_os(iter)`, `with_cache(bool)`, `with_workspace`, `features(list)`, `with_no_default_features_build`, `with_all_features_build`, `with_path_dep(PathDep)`, `with_clippy`, `with_fmt`, `with_docs`, `with_msrv(version)`, `target`, `target_kind`.
- New `PathDep` type representing a sibling path-dependency that the workflow must `git clone` into `../<name>`. Repeatable via `with_path_dep`. Matches the pattern the dev-* suite itself uses for CI.
- Generated `test` job emits a fail-fast=false matrix on the configured OS list with `Test (${{ matrix.os }})` step names, default `Build`/`Test` cargo steps, plus optional `Build (no default features)`, `Build (all features)`, and `Test (all features)` steps controlled by the corresponding builder methods.
- Generated `clippy` job runs `cargo clippy --all-targets --all-features -- -D warnings` followed by `cargo clippy --all-targets --no-default-features -- -D warnings`.
- Generated `fmt` job runs `cargo fmt --all -- --check`.
- Generated `docs` job sets `RUSTDOCFLAGS="-D warnings"` and runs `cargo doc --all-features --no-deps`.
- Generated `msrv` job pins `dtolnay/rust-toolchain@<version>` and runs `cargo build`.
- `actions/checkout` is at `v5` in every generated job (Node 24-compatible per REPS § 4).
- `Swatinem/rust-cache@v2` is enabled by default and can be toggled off via `with_cache(false)`.
- Path-dep clones are emitted as a single `run: |` step listing every `git clone --depth 1 <url> ../<name>` invocation, immediately after `actions/checkout`.
- Deterministic output: same `Generator` configuration produces byte-identical YAML across runs and machines (REPS § 3).
- New CLI binary at `src/bin/dev_ci.rs`. The binary publishes as `dev-ci`. Surface: `dev-ci generate [OPTIONS]` with `--target`, `--output`, `--print`, `--workflow-name`, `--branches`, `--matrix`, `--with`, `--msrv`, `--features`, `--no-default-features-build`, `--all-features-build`, `--workspace`, `--no-cache`, `--path-dep`. Writes to `.github/workflows/ci.yml` by default, or to stdout when `--print` (or `--output -`) is set.
- Examples: `basic.rs`, `full_suite.rs` (3-OS matrix + every job), `path_deps.rs` (sibling-clone pattern), `write_to_disk.rs` (file output with safe default path).
- 22 unit tests covering: header, branches filter, matrix, cache toggle, workspace flag, features list, no-default / all-features builds, path-dep clone step shape, clippy / fmt / docs / msrv job emission, output determinism, full-kitchen-sink combination, `PathDep` accessors, target kind default.
- 3 CLI unit tests covering `--path-dep` parsing (split on `=`, reject missing `=`, reject empty halves).

### Changed

- README rewritten: removes the "subprocess integration lands in 0.9.1" placeholder language, documents every builder method, the CLI surface, and the determinism guarantee. Pins MSRV at 1.85.
- REPS.md tightened: the "SHOULD provide" items (`workflow_name`/branches/matrix configuration, path-dep cloning, cache toggle) become MUST-have. The list of "later versions" items (GitLab, Buildkite, CircleCI, GitHub Action runtime) is preserved.
- CI workflow: new `cli` job builds the binary, invokes it with representative arguments, and grep-asserts the emitted YAML contains the expected markers. Sibling-clone of `../dev-report` in every job; `actions/checkout@v5` everywhere.

### Dependencies

- Added: `clap` 4 (with `derive` feature) for the CLI binary.

### Note

`0.1.0` was a name-claim publish with a minimal stub generator. The
public API additions in 0.9.0 are mostly additive: existing methods
(`new`, `target`, `with_clippy`, `with_fmt`, `with_msrv`, `generate`,
`target_kind`) keep their signatures and existing behavior. The
`Generator` struct gained private fields, so callers using the
builder pattern continue to compile unchanged.

[Unreleased]: https://github.com/jamesgober/dev-ci/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/jamesgober/dev-ci/releases/tag/v0.9.0
[0.1.0]: https://github.com/jamesgober/dev-ci/releases/tag/v0.1.0
