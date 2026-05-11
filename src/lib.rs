//! # dev-ci
//!
//! CI workflow generator and GitHub Action for the `dev-*` verification suite.
//!
//! `dev-ci` does two things:
//!
//! 1. **Generate** calibrated CI pipelines (`.github/workflows/ci.yml`,
//!    `.gitlab-ci.yml`, etc.) tailored to the dev-* features a project uses.
//! 2. **Run** the suite end-to-end on pull requests, posting structured
//!    annotations and uploading SARIF for the GitHub Security tab.
//!
//! ## Quick example
//!
//! ```no_run
//! use dev_ci::{Generator, Target};
//!
//! let yaml = Generator::new()
//!     .target(Target::GitHubActions)
//!     .with_clippy()
//!     .with_fmt()
//!     .with_msrv("1.85")
//!     .generate();
//!
//! std::fs::write(".github/workflows/ci.yml", yaml).unwrap();
//! ```
//!
//! ## Status
//!
//! Pre-1.0. APIs may shift through the `0.9.x` line. See the
//! [`CHANGELOG`](https://github.com/jamesgober/dev-ci/blob/main/CHANGELOG.md)
//! for what's stable.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

/// Supported CI target platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// GitHub Actions workflow YAML.
    GitHubActions,
}

/// Builder for a CI workflow document.
///
/// # Example
///
/// ```
/// use dev_ci::{Generator, Target};
///
/// let g = Generator::new().target(Target::GitHubActions);
/// assert_eq!(g.target_kind(), Target::GitHubActions);
/// ```
#[derive(Debug, Clone)]
pub struct Generator {
    target: Target,
    clippy: bool,
    fmt: bool,
    msrv: Option<String>,
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator {
    /// Begin a new generator with default settings.
    pub fn new() -> Self {
        Self {
            target: Target::GitHubActions,
            clippy: false,
            fmt: false,
            msrv: None,
        }
    }

    /// Select the target CI platform.
    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    /// Include a Clippy job.
    pub fn with_clippy(mut self) -> Self {
        self.clippy = true;
        self
    }

    /// Include a rustfmt check job.
    pub fn with_fmt(mut self) -> Self {
        self.fmt = true;
        self
    }

    /// Include an MSRV verification job pinned to the given Rust version.
    pub fn with_msrv(mut self, version: impl Into<String>) -> Self {
        self.msrv = Some(version.into());
        self
    }

    /// Selected target.
    pub fn target_kind(&self) -> Target {
        self.target
    }

    /// Render the workflow document.
    pub fn generate(&self) -> String {
        // 0.9.0 stub. Full implementation lands in 0.9.1+.
        match self.target {
            Target::GitHubActions => self.render_github_actions(),
        }
    }

    fn render_github_actions(&self) -> String {
        let mut out = String::new();
        out.push_str("name: CI\n\n");
        out.push_str(
            "on:\n  push:\n    branches: [main]\n  pull_request:\n    branches: [main]\n\n",
        );
        out.push_str("env:\n  CARGO_TERM_COLOR: always\n\n");
        out.push_str("jobs:\n");
        out.push_str("  test:\n    runs-on: ubuntu-latest\n    steps:\n");
        out.push_str("      - uses: actions/checkout@v5\n");
        out.push_str("      - uses: dtolnay/rust-toolchain@stable\n");
        out.push_str("      - run: cargo test --all-features\n");
        if self.clippy {
            out.push_str("  clippy:\n    runs-on: ubuntu-latest\n    steps:\n");
            out.push_str("      - uses: actions/checkout@v5\n");
            out.push_str(
                "      - uses: dtolnay/rust-toolchain@stable\n        with:\n          components: clippy\n",
            );
            out.push_str("      - run: cargo clippy --all-targets -- -D warnings\n");
        }
        if self.fmt {
            out.push_str("  fmt:\n    runs-on: ubuntu-latest\n    steps:\n");
            out.push_str("      - uses: actions/checkout@v5\n");
            out.push_str(
                "      - uses: dtolnay/rust-toolchain@stable\n        with:\n          components: rustfmt\n",
            );
            out.push_str("      - run: cargo fmt --all -- --check\n");
        }
        if let Some(msrv) = &self.msrv {
            out.push_str("  msrv:\n    runs-on: ubuntu-latest\n    steps:\n");
            out.push_str("      - uses: actions/checkout@v5\n");
            out.push_str(&format!("      - uses: dtolnay/rust-toolchain@{msrv}\n"));
            out.push_str("      - run: cargo build --all-features\n");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_generates_a_test_job() {
        let yaml = Generator::new().generate();
        assert!(yaml.contains("jobs:"));
        assert!(yaml.contains("test:"));
        assert!(yaml.contains("actions/checkout@v5"));
    }

    #[test]
    fn clippy_job_added_when_requested() {
        let yaml = Generator::new().with_clippy().generate();
        assert!(yaml.contains("clippy:"));
    }

    #[test]
    fn fmt_job_added_when_requested() {
        let yaml = Generator::new().with_fmt().generate();
        assert!(yaml.contains("fmt:"));
    }

    #[test]
    fn msrv_job_uses_pinned_toolchain() {
        let yaml = Generator::new().with_msrv("1.85").generate();
        assert!(yaml.contains("rust-toolchain@1.85"));
    }
}
