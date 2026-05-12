# dev-ci — Project Specification (REPS)

> Rust Engineering Project Specification.
> Normative language follows RFC 2119.

## 1. Purpose

`dev-ci` MUST provide two capabilities:

1. **Generation** — produce CI workflow files for popular platforms
   from a structured Rust API and a CLI front-end.
2. **Execution** — run the full `dev-*` verification suite end-to-end
   inside a CI job, emitting `dev-report::MultiReport` for programmatic
   consumption (planned for a later 0.9.x release).

Output MUST be machine-readable. AI agents and CI gates MUST be able
to consume the result and decide accept / reject / retry / escalate
without parsing free-form logs.

## 2. Scope

This crate MUST provide:

- A `Target` enum naming supported CI platforms.
- A `Generator` builder with the full standard surface: workflow name,
  branches filter, OS matrix, cache toggle, workspace flag, features
  list, no-default and all-features build steps, sibling path-dep
  cloning, `clippy`, `fmt`, `docs`, and `msrv` jobs.
- A `PathDep` value capturing the `(name, repo_url)` pair for a sibling
  path-dependency.
- A CLI binary (`dev-ci`) wrapping the library for terminal use. The
  CLI surface MUST cover every builder method.

This crate MAY provide later:

- GitLab CI YAML output.
- Buildkite YAML output.
- CircleCI YAML output.
- A GitHub Action distribution that wraps the runtime side (runs the
  full dev-* suite end-to-end and uploads SARIF).
- PR comment posting via GitHub API.

This crate MUST NOT:

- Replace the underlying CI runners. It generates configuration; it
  does not host runners.
- Encode opinions specific to one organization. Templates MUST be
  parametric.
- Embed credentials. All authentication is handled by the CI
  platform's standard mechanisms.

## 3. Determinism

A given `Generator` configuration MUST produce byte-identical YAML
across runs and across machines. No clock reads, no random IDs.
Iteration over user-supplied lists (branches, matrix OS, path-deps)
MUST follow insertion order. This is required for diff-based review
of CI changes.

Two calls to `generate()` on the same `Generator` instance MUST
return equal strings.

## 4. Target platforms

`Target::GitHubActions` MUST emit:

- `name`, `on`, `env` headers.
- One `test` job by default. The `test` job MUST be a matrix job
  with `fail-fast: false` and the configured OS list.
- Optional jobs for `clippy`, `fmt`, `docs`, `msrv` as configured.
- All `actions/checkout` references at `v5` or later (Node 24-compatible).
- `Swatinem/rust-cache@v2` after the toolchain setup, when caching is
  enabled (the default).
- When path-deps are declared, a `Check out sibling crates (path deps)`
  step immediately after the checkout step that emits
  `git clone --depth 1 <url> ../<name>` lines for each declared
  `PathDep`.

Other targets (GitLab, Buildkite, CircleCI) MUST be feature-gated or
fully optional dependencies in any future release.

## 5. Integration with dev-report

When the runtime execution side ships (planned for a later 0.9.x
release), the runner MUST emit a `dev-report::MultiReport` aggregating
every producer it invoked. The report SHOULD be:

- Written to disk at a configurable path.
- Uploaded as a CI artifact.
- Optionally converted to SARIF for the GitHub Security tab.

## 6. CLI contract

The `dev-ci` binary MUST expose every builder method through CLI flags.
The exit code MUST be:

- `0` on successful generation.
- non-zero with a single-line `dev-ci: <message>` error on stderr for
  malformed arguments, invalid `--path-dep` syntax, missing required
  fields (e.g. `--with msrv` without `--msrv`), or filesystem failures.

The CLI MUST support writing to stdout (`--print` or `--output -`) as
well as to a file. When writing to a file, parent directories MUST be
created as needed.

## 7. Stability

Through `0.9.x`, the public Rust API MAY shift between minor versions
but generated YAML for a given configuration MUST remain stable across
patch releases. The `1.0` release pins the Rust API; YAML output is
treated as versioned data and continues to evolve through minor
releases beyond 1.0.
