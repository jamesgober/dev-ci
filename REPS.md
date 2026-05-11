# dev-ci — Project Specification (REPS)

> Rust Engineering Project Specification.
> Normative language follows RFC 2119.

## 1. Purpose

`dev-ci` MUST provide two capabilities:

1. **Generation** — produce CI workflow files for popular platforms
   from a structured Rust API and a CLI front-end.
2. **Execution** — run the full `dev-*` verification suite end-to-end
   inside a CI job, emitting `dev-report::MultiReport` for
   programmatic consumption.

Output MUST be machine-readable. AI agents and CI gates MUST be able
to consume the result and decide accept / reject / retry / escalate
without parsing free-form logs.

## 2. Scope

This crate MUST provide:

- A `Target` enum naming supported CI platforms.
- A `Generator` builder for workflow construction.
- A CLI binary (`dev-ci`) wrapping the library for terminal use.

This crate SHOULD provide (later versions):

- GitLab CI YAML output.
- Buildkite YAML output.
- CircleCI YAML output.
- A GitHub Action distribution that wraps the binary.
- SARIF upload integration.
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
across runs and across machines. This is required for diff-based
review of CI changes.

## 4. Target platforms

`Target::GitHubActions` MUST emit:

- `name`, `on`, `env` headers.
- One `test` job by default.
- Optional jobs for `clippy`, `fmt`, `msrv`, `docs` as configured.
- All `actions/checkout` references at `v5` or later (Node 24-compatible).

Other targets MUST be feature-gated or fully optional dependencies.

## 5. Integration with dev-report

When the runtime side ships, the runner MUST emit a
`dev-report::MultiReport` aggregating every producer it invoked. The
report SHOULD be:

- Written to disk at a configurable path.
- Uploaded as a CI artifact.
- Optionally converted to SARIF for the GitHub Security tab.

## 6. Stability

Through `0.9.x`, the public Rust API MAY shift between minor versions
but generated YAML for a given configuration MUST remain stable.
The `1.0` release pins the Rust API; YAML output is treated as
versioned data and continues to evolve through minor releases.
