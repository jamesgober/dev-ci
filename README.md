<h1 align="center">
    <strong>dev-ci</strong>
    <br>
    <sup><sub>CI WORKFLOW GENERATOR AND GITHUB ACTION</sub></sup>
</h1>

<p align="center">
    <a href="https://crates.io/crates/dev-ci"><img alt="crates.io" src="https://img.shields.io/crates/v/dev-ci.svg"></a>
    <a href="https://crates.io/crates/dev-ci"><img alt="downloads" src="https://img.shields.io/crates/d/dev-ci.svg"></a>
    <a href="https://docs.rs/dev-ci"><img alt="docs.rs" src="https://docs.rs/dev-ci/badge.svg"></a>
    <a href="https://github.com/jamesgober/dev-ci/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/jamesgober/dev-ci/actions/workflows/ci.yml/badge.svg"></a>
</p>

<p align="center">
    CI workflow generator and GitHub Action for the <code>dev-*</code> verification suite.
</p>

---

## What it does

`dev-ci` solves two problems in one crate:

1. **Generates** calibrated CI workflow files (`.github/workflows/ci.yml`,
   GitLab CI, others later) tailored to the `dev-*` features a project
   actually uses. No more copy-pasting outdated CI templates from Stack
   Overflow.
2. **Runs** the entire `dev-*` verification suite end-to-end inside a
   single GitHub Action step, with structured PR annotations and SARIF
   upload to the GitHub Security tab.

## Why it exists

Setting up CI for a Rust project that uses test coverage, security
auditing, fuzzing, mutation testing, and async validation today means
hand-rolling 200+ lines of YAML across multiple jobs. `dev-ci` reduces
that to one workflow step and one source of truth.

## Quick start (CLI generator)

```bash
cargo install dev-ci
cd my-rust-project
dev-ci generate --target github-actions --with clippy,fmt,msrv
```

This writes a properly calibrated `.github/workflows/ci.yml` based on
what's in your `Cargo.toml` and your `dev-tools` feature set.

## Quick start (Library usage)

```toml
[dependencies]
dev-ci = "0.9"
```

```rust
use dev_ci::{Generator, Target};

let yaml = Generator::new()
    .target(Target::GitHubActions)
    .with_clippy()
    .with_fmt()
    .with_msrv("1.85")
    .generate();

std::fs::write(".github/workflows/ci.yml", yaml)?;
# Ok::<(), std::io::Error>(())
```

## Quick start (GitHub Action)

In `.github/workflows/ci.yml`:

```yaml
- name: Run dev-* suite
  uses: jamesgober/dev-ci@v1
  with:
    features: full
```

That's it. The action runs every enabled producer (`dev-bench`,
`dev-coverage`, `dev-security`, etc.), uploads SARIF for code scanning,
and posts a one-line PR status with a link to the full meta-report.

## The `dev-*` suite

`dev-ci` is part of the wider `dev-*` verification suite:

- [`dev-report`](https://github.com/jamesgober/dev-report) - foundation schema
- [`dev-tools`](https://github.com/jamesgober/dev-tools) - umbrella crate
- [`dev-fixtures`](https://github.com/jamesgober/dev-fixtures) - test environments
- [`dev-bench`](https://github.com/jamesgober/dev-bench) - performance
- [`dev-async`](https://github.com/jamesgober/dev-async) - async validation
- [`dev-stress`](https://github.com/jamesgober/dev-stress) - load testing
- [`dev-chaos`](https://github.com/jamesgober/dev-chaos) - failure injection
- [`dev-coverage`](https://github.com/jamesgober/dev-coverage) - test coverage
- [`dev-security`](https://github.com/jamesgober/dev-security) - vulnerability scanning
- [`dev-deps`](https://github.com/jamesgober/dev-deps) - dependency health
- [`dev-fuzz`](https://github.com/jamesgober/dev-fuzz) - fuzzing
- [`dev-flaky`](https://github.com/jamesgober/dev-flaky) - flaky test detection
- [`dev-mutate`](https://github.com/jamesgober/dev-mutate) - mutation testing

## Status

`v0.9.x` is pre-1.0. The generator works for basic GitHub Actions
workflows. Multi-target support (GitLab, Buildkite, CircleCI), the
GitHub Action runtime, and the PR-annotation pipeline land
through the `0.9.x` line.

## Minimum supported Rust version

`1.85` — pinned in `Cargo.toml` and verified by CI.

## License

Apache-2.0. See [LICENSE](LICENSE).
