<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br>
    <strong>dev-ci</strong>
    <br>
    <sup><sub>CI ORCHESTRATION FOR RUST CRATES</sub></sup>
</h1>
<p align="center">
    <a href="https://crates.io/crates/dev-ci"><img alt="crates.io" src="https://img.shields.io/crates/v/dev-ci.svg"></a>
    <a href="https://crates.io/crates/dev-ci"><img alt="downloads" src="https://img.shields.io/crates/d/dev-ci.svg"></a>
    <a href="https://github.com/jamesgober/dev-ci/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/jamesgober/dev-ci/actions/workflows/ci.yml/badge.svg"></a>
    <img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.85%2B-blue.svg?style=flat-square" title="Rust Version">
    <a href="https://docs.rs/dev-ci"><img alt="docs.rs" src="https://docs.rs/dev-ci/badge.svg"></a>
</p>

<p align="center">
    <strong>Generate calibrated GitHub Actions workflows for Rust crates.</strong> Library API <em>plus</em> CLI binary. Byte-deterministic output. <code>actions/checkout@v5</code> pinned everywhere.
</p>

<br>

<div align="center">
    <strong>Part of the <a href="https://crates.io/crates/dev-tools"><code>dev-*</code></a> verification collection.</strong><br>
    <sub>Also available as the <code>ci</code> feature of the <a href="https://crates.io/crates/dev-tools"><code>dev-tools</code></a> umbrella crate &mdash; one dependency, every verification layer.</sub>
</div>

<br>

---

## What it does

`dev-ci` generates calibrated CI workflow files (currently GitHub
Actions; other platforms in the roadmap) from a structured Rust API
plus a CLI front-end. One source of truth, byte-deterministic output,
every checkout pinned at `actions/checkout@v5`.

The runtime side — a GitHub Action that runs the entire `dev-*`
verification collection end-to-end inside one job and uploads SARIF —
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

assert!(yaml.contains("git clone --depth 1 'https://github.com/jamesgober/dev-report.git' '../dev-report'"));
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

`dev-ci` ships a CLI binary as well as the library. Install it once:

```bash
cargo install dev-ci
```

Then `dev-ci --help` prints the full reference; the patterns below
cover the common use cases.

### Install

```bash
cargo install dev-ci                 # latest release from crates.io
cargo install --git https://github.com/jamesgober/dev-ci   # tip of main
```

### One-shot, accept the defaults

```bash
dev-ci generate
```

Writes `.github/workflows/ci.yml` with: `actions/checkout@v5`, a
single `test` job on `ubuntu-latest`, `cargo build`, `cargo test`,
and the `Swatinem/rust-cache@v2` action enabled.

### Multi-OS matrix with the standard quality jobs

```bash
dev-ci generate \
    --matrix ubuntu-latest,macos-latest,windows-latest \
    --with clippy,fmt,docs,msrv \
    --msrv 1.85
```

You get `test` (3 OSes) plus `clippy`, `fmt`, `docs`, and `msrv` as
their own jobs.

### Crate that uses path-deps to sibling repos

```bash
dev-ci generate \
    --features fixtures,bench,coverage \
    --path-dep dev-report=https://github.com/jamesgober/dev-report.git \
    --path-dep dev-fixtures=https://github.com/jamesgober/dev-fixtures.git \
    --path-dep dev-bench=https://github.com/jamesgober/dev-bench.git
```

Each `--path-dep` adds a `git clone --depth 1` step that runs before
`cargo`, dropping the sibling into `..` so the path-dep resolves.

### Preview without writing

```bash
dev-ci generate --print            # send YAML to stdout
dev-ci generate --output -         # alias for --print
```

Pipe into `kdiff3`, `delta`, or your editor when you're not ready to
overwrite the existing workflow.

### Custom output path or workflow name

```bash
dev-ci generate \
    --output .github/workflows/quality.yml \
    --workflow-name "Quality Gates"
```

### Workspace project

```bash
dev-ci generate --workspace
```

Adds `--workspace` to every cargo call in the workflow.

### Restrict trigger branches

```bash
dev-ci generate --branches main,release/*
```

Both `push` and `pull_request` are filtered to the matching branches.

### Disable the cache action

```bash
dev-ci generate --no-cache
```

Omits the `Swatinem/rust-cache@v2` step. Use when the cache layer
itself is what you're trying to debug.

### Flag reference

| Flag                              | Equivalent builder method                  |
|-----------------------------------|--------------------------------------------|
| `--target github-actions`         | `target(Target::GitHubActions)` (default)  |
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

### Exit codes

| Code | Meaning                                            |
|:---:|----------------------------------------------------|
| `0` | Workflow generated and written successfully.       |
| `1` | Bad arguments, unknown job in `--with`, or I/O error writing the output file. The reason is printed to stderr. |

### Combine with `--print` for review workflows

The CLI is deterministic, so it composes well with code review:

```bash
# Diff against what's checked in
dev-ci generate --print | diff -u .github/workflows/ci.yml -

# Apply only if the diff looks right
dev-ci generate                       # writes the file
git add .github/workflows/ci.yml
```

## Examples

| File                              | What it shows                                                       |
|-----------------------------------|---------------------------------------------------------------------|
| `examples/basic.rs`               | Minimal workflow — single-OS test + clippy + fmt + docs + msrv.     |
| `examples/full_suite.rs`          | 3-OS matrix, every job, workspace + all-features + no-default builds. |
| `examples/path_deps.rs`           | The sibling-clone pattern the dev-* suite itself uses.              |
| `examples/write_to_disk.rs`       | Generate and write the workflow to a file.                          |

## The `dev-*` collection

`dev-ci` ships independently and is also re-exported by the
[`dev-tools`](https://crates.io/crates/dev-tools) umbrella crate as
the `ci` feature. Sister crates cover the other verification
dimensions:

- [`dev-report`](https://crates.io/crates/dev-report) &mdash; report schema everything emits
- [`dev-fixtures`](https://crates.io/crates/dev-fixtures) &mdash; deterministic test fixtures
- [`dev-bench`](https://crates.io/crates/dev-bench) &mdash; performance and regression detection
- [`dev-async`](https://crates.io/crates/dev-async) &mdash; async runtime verification
- [`dev-stress`](https://crates.io/crates/dev-stress) &mdash; stress and soak workloads
- [`dev-chaos`](https://crates.io/crates/dev-chaos) &mdash; fault injection and recovery testing
- [`dev-coverage`](https://crates.io/crates/dev-coverage) &mdash; code coverage with regression gates
- [`dev-security`](https://crates.io/crates/dev-security) &mdash; CVE / license / banned-crate audit
- [`dev-deps`](https://crates.io/crates/dev-deps) &mdash; unused / outdated dep detection
- [`dev-fuzz`](https://crates.io/crates/dev-fuzz) &mdash; fuzz testing workflow
- [`dev-flaky`](https://crates.io/crates/dev-flaky) &mdash; flaky-test detection
- [`dev-mutate`](https://crates.io/crates/dev-mutate) &mdash; mutation testing

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




<!-- COPYRIGHT
---------------------------------->
<div align="center">
    <br>
    <h2></h2>
    Copyright &copy; 2026 James Gober.
</div>
