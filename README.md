<h1 align="center">
    <strong>dev-ci</strong>
    <br>
    <sup><sub>CI WORKFLOW GENERATOR FOR RUST</sub></sup>
</h1>

<p align="center">
    <a href="https://crates.io/crates/dev-ci"><img alt="crates.io" src="https://img.shields.io/crates/v/dev-ci.svg"></a>
    <a href="https://crates.io/crates/dev-ci"><img alt="downloads" src="https://img.shields.io/crates/d/dev-ci.svg"></a>
    <a href="https://docs.rs/dev-ci"><img alt="docs.rs" src="https://docs.rs/dev-ci/badge.svg"></a>
    <a href="https://github.com/jamesgober/dev-ci/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/jamesgober/dev-ci/actions/workflows/ci.yml/badge.svg"></a>
    <img alt="MSRV" src="https://img.shields.io/badge/msrv-1.85%2B-blue.svg?style=flat-square" title="Rust Version">
</p>

<p align="center">
    CI workflow generator for the <code>dev-*</code> verification suite.
</p>

---

## What it does

`dev-ci` generates calibrated CI workflow files (currently GitHub
Actions; others in the roadmap) from a structured Rust API and a CLI
front-end. One source of truth, byte-deterministic output, every
checkout pinned at `actions/checkout@v5`.

The runtime side — a GitHub Action that runs the entire `dev-*`
verification suite end-to-end inside one job and uploads SARIF —
lands later in the 0.9.x line.

## Quick start (CLI)

```bash
cargo install dev-ci
cd my-rust-project
dev-ci generate \
    --with clippy,fmt,docs,msrv \
    --msrv 1.85 \
    --matrix ubuntu-latest,macos-latest,windows-latest
```

Writes `.github/workflows/ci.yml`. Pass `--print` to send the result
to stdout instead.

## Quick start (Library)

```toml
[dependencies]
dev-ci = "0.9"
```

```rust
use dev_ci::{Generator, Target};

let yaml = Generator::new()
    .target(Target::GitHubActions)
    .matrix_os(["ubuntu-latest", "macos-latest", "windows-latest"])
    .with_clippy()
    .with_fmt()
    .with_docs()
    .with_msrv("1.85")
    .generate();

std::fs::write(".github/workflows/ci.yml", yaml).unwrap();
```

## Builder surface

| Method                            | What it does                                                        |
|-----------------------------------|---------------------------------------------------------------------|
| `target(Target)`                  | Pick the output platform. Default: `GitHubActions`.                 |
| `workflow_name(s)`                | Set the `name:` field. Default: `"CI"`.                             |
| `branches(iter)`                  | Branch filter for both `push` and `pull_request`. Default: `["main"]`. |
| `matrix_os(iter)`                 | OS list for the `test` matrix. Default: `["ubuntu-latest"]`.        |
| `with_cache(bool)`                | Toggle `Swatinem/rust-cache@v2`. Default: enabled.                  |
| `with_workspace()`                | Pass `--workspace` to every cargo call.                             |
| `features(list)`                  | Pass `--features <list>` to default build/test steps.               |
| `with_no_default_features_build()`| Add a `cargo build --no-default-features` step.                     |
| `with_all_features_build()`       | Add `cargo build --all-features` + `cargo test --all-features`.     |
| `with_path_dep(PathDep)`          | Declare a sibling path-dep to `git clone` before cargo runs.        |
| `with_clippy()`                   | Add a `clippy` job (all-features + no-default-features lint runs).  |
| `with_fmt()`                      | Add a `fmt` job (`cargo fmt --all -- --check`).                     |
| `with_docs()`                     | Add a `docs` job with `RUSTDOCFLAGS="-D warnings"`.                 |
| `with_msrv(version)`              | Add an `msrv` job pinning `dtolnay/rust-toolchain@<version>`.       |

## Sibling path-deps

For the dev-* suite itself (and any other multi-crate-repo project)
each crate's CI clones its siblings into `..` before running cargo:

```rust
use dev_ci::{Generator, PathDep};

let yaml = Generator::new()
    .with_path_dep(PathDep::new("dev-report", "https://github.com/jamesgober/dev-report.git"))
    .with_path_dep(PathDep::new("dev-tools",  "https://github.com/jamesgober/dev-tools.git"))
    .generate();

assert!(yaml.contains("git clone --depth 1 https://github.com/jamesgober/dev-report.git ../dev-report"));
```

The clones land in a single `run: |` step right after
`actions/checkout`, before the toolchain install.

## Determinism

A given `Generator` configuration produces byte-identical YAML across
runs and machines — no clock reads, no random IDs, list iteration is
in insertion order. Two calls to `generate()` on the same `Generator`
return equal strings.

This makes the generated YAML safe to diff during code review and
safe to assert against in unit tests.

## CLI

```text
dev-ci generate [OPTIONS]
```

| Flag                              | Equivalent builder method                  |
|-----------------------------------|--------------------------------------------|
| `--target github-actions`         | `target(Target::GitHubActions)`            |
| `--workflow-name <NAME>`          | `workflow_name(NAME)`                      |
| `--branches main,release/*`       | `branches([...])`                          |
| `--matrix ubuntu-latest,macos-latest` | `matrix_os([...])`                     |
| `--with clippy,fmt,docs,msrv`     | `with_clippy()` / `with_fmt()` / `with_docs()` / `with_msrv(...)` |
| `--msrv 1.85`                     | `with_msrv("1.85")`                        |
| `--features foo,bar`              | `features("foo,bar")`                      |
| `--no-default-features-build`     | `with_no_default_features_build()`         |
| `--all-features-build`            | `with_all_features_build()`                |
| `--workspace`                     | `with_workspace()`                         |
| `--no-cache`                      | `with_cache(false)`                        |
| `--path-dep name=url`             | `with_path_dep(PathDep::new(name, url))`   |
| `--output <PATH>` / `--print`     | Write to file (default `.github/workflows/ci.yml`) or stdout. |

## Examples

| File                              | What it shows                                                       |
|-----------------------------------|---------------------------------------------------------------------|
| `examples/basic.rs`               | Minimal workflow — single-OS test + clippy + fmt + docs + msrv.     |
| `examples/full_suite.rs`          | 3-OS matrix, every job, workspace + all-features + no-default builds. |
| `examples/path_deps.rs`           | The sibling-clone pattern the dev-* suite itself uses.              |
| `examples/write_to_disk.rs`       | Generate and write the workflow to a file.                          |

## The `dev-*` suite

`dev-ci` is part of the wider `dev-*` verification suite:

- [`dev-report`](https://github.com/jamesgober/dev-report) — foundation schema
- [`dev-tools`](https://github.com/jamesgober/dev-tools) — umbrella crate
- [`dev-fixtures`](https://github.com/jamesgober/dev-fixtures) — test environments
- [`dev-bench`](https://github.com/jamesgober/dev-bench) — performance
- [`dev-async`](https://github.com/jamesgober/dev-async) — async validation
- [`dev-stress`](https://github.com/jamesgober/dev-stress) — load testing
- [`dev-chaos`](https://github.com/jamesgober/dev-chaos) — failure injection
- [`dev-coverage`](https://github.com/jamesgober/dev-coverage) — test coverage
- [`dev-security`](https://github.com/jamesgober/dev-security) — vulnerability scanning
- [`dev-deps`](https://github.com/jamesgober/dev-deps) — dependency health
- `dev-fuzz`, `dev-flaky`, `dev-mutate` — landing soon

## Status

`v0.9.x` is the pre-1.0 stabilization line. The generator and CLI are
feature-complete for GitHub Actions output. GitLab CI, Buildkite,
CircleCI targets, and the runtime GitHub Action distribution land in
subsequent 0.9.x releases.

## Minimum supported Rust version

`1.85` — pinned in `Cargo.toml` via `rust-version` and verified by
the MSRV job in CI.

## License

Apache-2.0. See [LICENSE](LICENSE).
