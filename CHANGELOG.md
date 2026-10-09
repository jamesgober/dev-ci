# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.3] - 2026-10-09

Library / binary MSRV split, plus YAML and shell quoting fixes from a
review pass. Library MSRV drops to 1.75; the binary stays at 1.85.

### Added

- `VERSION` constant with the crate version as compiled, so tools that
  bundle this crate can report what is actually linked.

### Fixed

- Generated YAML quoting:
  - Matrix entries were written unquoted, so a value like `x, y` broke
    the workflow.
  - Branches, names and matrix entries that YAML reads as a bool, null
    or number (`on`, `no`, `1.10` read as `1.1`) are now quoted, and
    values containing control characters are double-quoted.
  - The MSRV value went unquoted into `name:` and `uses:`, so it could
    inject YAML.
  - `run:`, `uses:` and job-name values are quoted only when YAML would
    misread them; plain ones such as `cargo build --verbose` stay as
    they were.
- A `--features` value with spaces or shell metacharacters is now
  single-quoted for the shell, and that step runs under `shell: bash`.
  The path-dep clone step also runs under bash now; its POSIX quoting
  broke under PowerShell on Windows runners.
- `with_workspace` promised to reach every cargo invocation but skipped
  the clippy and doc jobs; both now get `--workspace`.
- CLI: `--msrv ""` produced `dtolnay/rust-toolchain@`. `--msrv` is now
  checked to be a toolchain name, and empty or padded entries in
  `--branches` and `--matrix` are trimmed and dropped.

### Changed

- **`clap` is now an optional dependency gated by a new `cli` feature.**
  The `Generator` + `PathDep` library API does not depend on clap and
  builds on Rust 1.75. The `dev-ci` binary still needs clap, and with it
  Rust 1.85 (clap 4.6+'s `clap_derive` / `clap_lex` moved to edition
  2024).
- **`[[bin]] required-features = ["cli"]`**, and **`default = ["cli"]`**
  keeps `cargo install dev-ci` working as before. Library users can
  stay on 1.75 with `dev-ci = { version = "0.9", default-features = false }`.
- `rust-version` lowered from `1.85` to `1.75` to reflect the library.
- The generated MSRV job first resolves a `rust-version`-compatible
  lockfile on stable (`CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback`)
  when the project has no committed `Cargo.lock`. Without it, the old
  toolchain picked the newest dependencies and could fail on ones that
  need a newer Rust.
- This repo's CI now checks both floors: the library on 1.75 and the
  binary on 1.85. The MSRV job was only building the binary.

### Documentation

- README: exit codes (0 success, 1 bad values or I/O error, 2 for a
  command-line parse error; it said parse errors exit 1), the MSRV
  split, the generated MSRV job, and the quoting rules.
- `docs/API.md` and the rustdoc for `with_msrv` and `with_path_dep`
  updated.

[0.9.3]: https://github.com/jamesgober/dev-ci/releases/tag/v0.9.3

## [0.9.2] - 2026-05-12

Hardening release surfaced by the post-polish audit pass.

### Fixed

- YAML scalar emission now single-quotes values that contain non-plain characters or start with a YAML indicator (`* & ? ! | > # @ : , [ ] { } %`). Previously `workflow_name("Build: CI")` would emit `name: Build: CI` and produce an invalid workflow file; it now emits `name: 'Build: CI'`. Same for branch names with commas, leading `*`, or other indicators.
- `git clone` commands generated for `--path-dep` arguments now POSIX-shell-quote both the URL and the target path. A repo URL containing a single quote (rare but possible in mirror paths) used to corrupt the generated `run:` block; it now round-trips correctly via the `'\\''` escape.

### Internal

- Two new helpers in `lib.rs`: `yaml_scalar()` (YAML plain-scalar safety check + single-quote fallback) and `shell_arg()` (POSIX shell single-quote with `'` escape).
- Five new tests cover the quoting behavior:
  - `yaml_scalar_quotes_workflow_name_with_colon`
  - `yaml_scalar_doubles_embedded_single_quote`
  - `yaml_scalar_quotes_branch_with_comma`
  - `yaml_scalar_quotes_branch_starting_with_indicator`
  - `shell_arg_escapes_embedded_single_quote`
- Existing `path_dep_clone_step_emitted` and `full_kitchen_sink_yaml_round_trip` tests updated for the now-quoted output.

### Notes for callers

If you previously asserted on the exact unquoted form of the generated YAML (e.g. `assert!(yaml.contains("git clone --depth 1 https://example.com/foo.git ../foo"))`), update to the quoted form: `assert!(yaml.contains("git clone --depth 1 'https://example.com/foo.git' '../foo'"))`. Plain branch names like `main` and `release/*` continue to emit unquoted.

[0.9.2]: https://github.com/jamesgober/dev-ci/releases/tag/v0.9.2

## [0.9.1] - 2026-05-12

Documentation polish and CLI cookbook. No code changes.

### Changed

- README header standardized: Rust logo image, MSRV badge between CI and docs.rs (was at the end of the badge list, lowercase label), copyright block at bottom.
- Subtitle now reads `CI ORCHESTRATION FOR RUST CRATES` (was `CI WORKFLOW GENERATOR FOR RUST`).
- Tagline rewritten to lead with the developer outcome — calibrated workflows, library + CLI, byte-deterministic output.
- `## CLI` section significantly expanded: install instructions, eight named usage patterns (defaults, multi-OS matrix, path-dep wiring, preview-without-write, custom output, workspace, branch filter, no-cache), exit-code reference, and a review-friendly `--print | diff` pattern.
- `## The dev-* suite` retitled to `The dev-* collection`; the stale `dev-fuzz, dev-flaky, dev-mutate — landing soon` line replaced with their actual crate links (all three shipped).
- `Cargo.toml` description rewritten: leads with the developer outcome (calibrated GitHub Actions workflow), names what the crate ships (library + CLI), highlights determinism.
- `Cargo.toml` keywords retuned: dropped `verification` and `ai-tools`, added `generator` and `rust` for crates.io search.

### Added

- "Part of the `dev-*` verification collection" block on the README, under the intro, linking the umbrella `dev-tools` crate.

[0.9.1]: https://github.com/jamesgober/dev-ci/releases/tag/v0.9.1

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
