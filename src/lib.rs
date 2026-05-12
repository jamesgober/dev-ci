//! # dev-ci
//!
//! CI workflow generator for the `dev-*` verification suite.
//!
//! `dev-ci` does two things:
//!
//! 1. **Generate** calibrated CI pipelines (`.github/workflows/ci.yml`,
//!    others to follow) tailored to the dev-* features a project uses.
//! 2. **Run** the suite end-to-end on pull requests (planned for later
//!    in the 0.9.x line — the runtime side ships as a GitHub Action).
//!
//! ## Quick example
//!
//! ```
//! use dev_ci::{Generator, Target};
//!
//! let yaml = Generator::new()
//!     .target(Target::GitHubActions)
//!     .with_clippy()
//!     .with_fmt()
//!     .with_docs()
//!     .with_msrv("1.85")
//!     .generate();
//!
//! assert!(yaml.contains("actions/checkout@v5"));
//! ```
//!
//! ## Determinism
//!
//! Output is byte-deterministic for a given [`Generator`] configuration.
//! No clock reads, no random IDs, lists iterate in insertion order.
//!
//! ## What's in 0.9.0
//!
//! - GitHub Actions output (every job uses `actions/checkout@v5`,
//!   `Swatinem/rust-cache@v2`, and the documented patterns from the
//!   existing dev-* suite CI).
//! - Builder methods for the full standard surface: `test` matrix,
//!   feature toggles, `clippy`, `fmt`, `docs`, `msrv`, sibling
//!   path-dep cloning, cache toggle, custom workflow name + branches.
//! - CLI binary (`dev-ci generate ...`) wrapping the library.
//!
//! Other targets (GitLab, Buildkite, CircleCI) and the runtime-action
//! side are planned for later 0.9.x releases.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::fmt::Write as _;

// ---------------------------------------------------------------------------
// Target
// ---------------------------------------------------------------------------

/// Supported CI target platforms.
///
/// Only [`Target::GitHubActions`] is implemented in 0.9.0; other
/// targets (GitLab CI, Buildkite, CircleCI) land in subsequent 0.9.x
/// releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// GitHub Actions workflow YAML.
    GitHubActions,
}

// ---------------------------------------------------------------------------
// PathDep
// ---------------------------------------------------------------------------

/// A sibling path-dependency that the CI workflow should `git clone`
/// before running cargo.
///
/// This matches the pattern the existing dev-* suite uses: each
/// crate's CI clones its siblings into `../<name>` so path-deps
/// resolve cleanly under a sibling-only checkout.
///
/// # Example
///
/// ```
/// use dev_ci::PathDep;
///
/// let dep = PathDep::new("dev-report", "https://github.com/jamesgober/dev-report.git");
/// assert_eq!(dep.name(), "dev-report");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathDep {
    name: String,
    repo_url: String,
}

impl PathDep {
    /// Build a path-dep descriptor.
    pub fn new(name: impl Into<String>, repo_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            repo_url: repo_url.into(),
        }
    }

    /// Sibling directory name (cloned under `../<name>`).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// HTTPS / SSH clone URL.
    pub fn repo_url(&self) -> &str {
        &self.repo_url
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Builder for a CI workflow document.
///
/// Methods are pure setters; configuration is committed only when
/// [`generate`](Self::generate) is called. Calling `generate` multiple
/// times against the same `Generator` returns byte-identical output.
///
/// # Example
///
/// ```
/// use dev_ci::{Generator, PathDep, Target};
///
/// let yaml = Generator::new()
///     .target(Target::GitHubActions)
///     .workflow_name("CI")
///     .branches(["main", "release/*"])
///     .matrix_os(["ubuntu-latest", "macos-latest", "windows-latest"])
///     .with_clippy()
///     .with_fmt()
///     .with_docs()
///     .with_msrv("1.85")
///     .with_no_default_features_build()
///     .with_all_features_build()
///     .with_path_dep(PathDep::new("dev-report", "https://github.com/jamesgober/dev-report.git"))
///     .generate();
///
/// assert!(yaml.contains("name: CI"));
/// assert!(yaml.contains("actions/checkout@v5"));
/// ```
#[derive(Debug, Clone)]
pub struct Generator {
    target: Target,
    workflow_name: String,
    branches: Vec<String>,
    matrix_os: Vec<String>,
    rust_cache: bool,
    workspace: bool,
    features: Option<String>,
    no_default_features_build: bool,
    all_features_build: bool,
    path_deps: Vec<PathDep>,
    clippy: bool,
    fmt: bool,
    docs: bool,
    msrv: Option<String>,
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator {
    /// Begin a new generator with default settings.
    ///
    /// Defaults: target = `GitHubActions`, workflow name = `"CI"`,
    /// branches = `["main"]`, matrix = `["ubuntu-latest"]`, cache
    /// enabled, no extra jobs.
    pub fn new() -> Self {
        Self {
            target: Target::GitHubActions,
            workflow_name: "CI".into(),
            branches: vec!["main".into()],
            matrix_os: vec!["ubuntu-latest".into()],
            rust_cache: true,
            workspace: false,
            features: None,
            no_default_features_build: false,
            all_features_build: false,
            path_deps: Vec::new(),
            clippy: false,
            fmt: false,
            docs: false,
            msrv: None,
        }
    }

    /// Select the target CI platform.
    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    /// Selected target.
    pub fn target_kind(&self) -> Target {
        self.target
    }

    /// Set the workflow `name:` field. Defaults to `"CI"`.
    pub fn workflow_name(mut self, name: impl Into<String>) -> Self {
        self.workflow_name = name.into();
        self
    }

    /// Set the `push` / `pull_request` branch filter list.
    ///
    /// Defaults to `["main"]`. Glob patterns are passed through unchanged.
    pub fn branches<I, S>(mut self, branches: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.branches = branches.into_iter().map(Into::into).collect();
        if self.branches.is_empty() {
            self.branches.push("main".into());
        }
        self
    }

    /// Set the OS matrix for the `test` job. Defaults to a single
    /// `ubuntu-latest` runner.
    ///
    /// Common multi-OS configuration:
    /// `["ubuntu-latest", "macos-latest", "windows-latest"]`.
    pub fn matrix_os<I, S>(mut self, os_list: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.matrix_os = os_list.into_iter().map(Into::into).collect();
        if self.matrix_os.is_empty() {
            self.matrix_os.push("ubuntu-latest".into());
        }
        self
    }

    /// Toggle the `Swatinem/rust-cache@v2` action. Default: enabled.
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.rust_cache = enabled;
        self
    }

    /// Pass `--workspace` to every cargo invocation.
    pub fn with_workspace(mut self) -> Self {
        self.workspace = true;
        self
    }

    /// Pass `--features <list>` to every cargo invocation.
    ///
    /// Mutually exclusive with [`with_all_features_build`](Self::with_all_features_build)
    /// only in the sense that `--all-features` overrides `--features`
    /// in cargo itself; the generator emits both flags as configured.
    pub fn features(mut self, features: impl Into<String>) -> Self {
        self.features = Some(features.into());
        self
    }

    /// Emit an additional build step under the `test` job that passes
    /// `--no-default-features`. Useful for crates with optional
    /// features that should compile cleanly under the bare configuration.
    pub fn with_no_default_features_build(mut self) -> Self {
        self.no_default_features_build = true;
        self
    }

    /// Emit an additional build + test step under the `test` job that
    /// passes `--all-features`.
    pub fn with_all_features_build(mut self) -> Self {
        self.all_features_build = true;
        self
    }

    /// Declare a sibling path-dependency that the workflow must
    /// `git clone` into `../<name>` before running cargo.
    ///
    /// Matches the pattern the existing dev-* suite uses for its own
    /// CI (each crate clones its siblings into `..`). May be called
    /// repeatedly.
    pub fn with_path_dep(mut self, dep: PathDep) -> Self {
        self.path_deps.push(dep);
        self
    }

    /// Include a clippy job.
    pub fn with_clippy(mut self) -> Self {
        self.clippy = true;
        self
    }

    /// Include a rustfmt-check job.
    pub fn with_fmt(mut self) -> Self {
        self.fmt = true;
        self
    }

    /// Include a `cargo doc` job with `RUSTDOCFLAGS="-D warnings"`.
    pub fn with_docs(mut self) -> Self {
        self.docs = true;
        self
    }

    /// Include an MSRV job pinned to the given Rust version.
    pub fn with_msrv(mut self, version: impl Into<String>) -> Self {
        self.msrv = Some(version.into());
        self
    }

    /// Render the workflow document.
    ///
    /// Output is byte-deterministic for a given configuration.
    pub fn generate(&self) -> String {
        match self.target {
            Target::GitHubActions => self.render_github_actions(),
        }
    }

    // -----------------------------------------------------------------------
    // GitHub Actions renderer
    // -----------------------------------------------------------------------

    fn render_github_actions(&self) -> String {
        let mut out = String::with_capacity(2048);
        self.write_header(&mut out);
        out.push_str("jobs:\n");
        self.write_test_job(&mut out);
        if self.clippy {
            self.write_clippy_job(&mut out);
        }
        if self.fmt {
            self.write_fmt_job(&mut out);
        }
        if self.docs {
            self.write_docs_job(&mut out);
        }
        if let Some(msrv) = self.msrv.clone() {
            self.write_msrv_job(&mut out, &msrv);
        }
        out
    }

    fn write_header(&self, out: &mut String) {
        writeln!(out, "name: {}", yaml_scalar(&self.workflow_name)).unwrap();
        out.push('\n');
        out.push_str("on:\n");
        out.push_str("  push:\n");
        write_branch_list(out, "    ", &self.branches);
        out.push_str("  pull_request:\n");
        write_branch_list(out, "    ", &self.branches);
        out.push('\n');
        out.push_str("env:\n  CARGO_TERM_COLOR: always\n\n");
    }

    fn write_test_job(&self, out: &mut String) {
        out.push_str("  test:\n");
        out.push_str("    name: Test (${{ matrix.os }})\n");
        out.push_str("    runs-on: ${{ matrix.os }}\n");
        out.push_str("    strategy:\n");
        out.push_str("      fail-fast: false\n");
        out.push_str("      matrix:\n");
        out.push_str("        os: [");
        for (i, os) in self.matrix_os.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(os);
        }
        out.push_str("]\n");
        out.push_str("    steps:\n");
        self.write_common_setup(out);

        // Default build + test.
        self.write_cargo_step(out, "Build", "build", false, false);
        self.write_cargo_step(out, "Test", "test", false, false);

        if self.no_default_features_build {
            self.write_cargo_step(out, "Build (no default features)", "build", true, false);
        }
        if self.all_features_build {
            self.write_cargo_step(out, "Build (all features)", "build", false, true);
            self.write_cargo_step(out, "Test (all features)", "test", false, true);
        }
    }

    fn write_clippy_job(&self, out: &mut String) {
        out.push_str("\n  clippy:\n");
        out.push_str("    name: Clippy\n");
        out.push_str("    runs-on: ubuntu-latest\n");
        out.push_str("    steps:\n");
        self.write_common_setup_components(out, Some("clippy"), None);
        out.push_str("      - name: Clippy (all features)\n");
        out.push_str("        run: cargo clippy --all-targets --all-features -- -D warnings\n");
        out.push_str("      - name: Clippy (no default features)\n");
        out.push_str(
            "        run: cargo clippy --all-targets --no-default-features -- -D warnings\n",
        );
    }

    fn write_fmt_job(&self, out: &mut String) {
        out.push_str("\n  fmt:\n");
        out.push_str("    name: Rustfmt\n");
        out.push_str("    runs-on: ubuntu-latest\n");
        out.push_str("    steps:\n");
        out.push_str("      - uses: actions/checkout@v5\n");
        out.push_str("      - uses: dtolnay/rust-toolchain@stable\n");
        out.push_str("        with:\n");
        out.push_str("          components: rustfmt\n");
        out.push_str("      - run: cargo fmt --all -- --check\n");
    }

    fn write_docs_job(&self, out: &mut String) {
        out.push_str("\n  docs:\n");
        out.push_str("    name: Doc build\n");
        out.push_str("    runs-on: ubuntu-latest\n");
        out.push_str("    env:\n");
        out.push_str("      RUSTDOCFLAGS: \"-D warnings\"\n");
        out.push_str("    steps:\n");
        self.write_common_setup(out);
        out.push_str("      - run: cargo doc --all-features --no-deps\n");
    }

    fn write_msrv_job(&self, out: &mut String, msrv: &str) {
        writeln!(out, "\n  msrv:").unwrap();
        writeln!(out, "    name: MSRV (Rust {msrv})").unwrap();
        out.push_str("    runs-on: ubuntu-latest\n");
        out.push_str("    steps:\n");
        self.write_common_setup_components(out, None, Some(msrv));
        let extras = self.cargo_flags_string(true, false);
        writeln!(out, "      - run: cargo build{extras}").unwrap();
    }

    fn write_common_setup(&self, out: &mut String) {
        self.write_common_setup_components(out, None, None);
    }

    fn write_common_setup_components(
        &self,
        out: &mut String,
        component: Option<&str>,
        toolchain_pin: Option<&str>,
    ) {
        out.push_str("      - uses: actions/checkout@v5\n");
        if !self.path_deps.is_empty() {
            out.push_str("      - name: Check out sibling crates (path deps)\n");
            out.push_str("        run: |\n");
            for dep in &self.path_deps {
                let target = format!("../{}", dep.name);
                writeln!(
                    out,
                    "          git clone --depth 1 {} {}",
                    shell_arg(&dep.repo_url),
                    shell_arg(&target),
                )
                .unwrap();
            }
        }
        let toolchain = toolchain_pin.unwrap_or("stable");
        writeln!(out, "      - uses: dtolnay/rust-toolchain@{toolchain}").unwrap();
        if let Some(c) = component {
            out.push_str("        with:\n");
            writeln!(out, "          components: {c}").unwrap();
        }
        if self.rust_cache {
            out.push_str("      - uses: Swatinem/rust-cache@v2\n");
        }
    }

    fn write_cargo_step(
        &self,
        out: &mut String,
        name: &str,
        cmd: &str,
        no_default_features: bool,
        all_features: bool,
    ) {
        let flags = self.cargo_flags_string(!no_default_features && !all_features, false);
        let extra = if no_default_features {
            " --no-default-features".to_string()
        } else if all_features {
            " --all-features".to_string()
        } else {
            String::new()
        };
        writeln!(out, "      - name: {name}").unwrap();
        writeln!(out, "        run: cargo {cmd}{flags}{extra} --verbose").unwrap();
    }

    /// Builds a flag suffix string (e.g. " --workspace --features foo").
    ///
    /// `include_features` controls whether `--features <list>` is
    /// emitted (suppressed when the caller already passes
    /// `--no-default-features` or `--all-features` so cargo doesn't
    /// fight the user). `force_workspace` lets a caller force the
    /// workspace flag even when the generator's default is off.
    fn cargo_flags_string(&self, include_features: bool, force_workspace: bool) -> String {
        let mut s = String::new();
        if self.workspace || force_workspace {
            s.push_str(" --workspace");
        }
        if include_features {
            if let Some(f) = &self.features {
                if !f.is_empty() {
                    s.push_str(" --features ");
                    s.push_str(f);
                }
            }
        }
        s
    }
}

fn write_branch_list(out: &mut String, indent: &str, branches: &[String]) {
    out.push_str(indent);
    out.push_str("branches: [");
    for (i, b) in branches.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&yaml_scalar(b));
    }
    out.push_str("]\n");
}

/// Emit a YAML plain scalar if the input is unambiguous, otherwise
/// single-quote it (with `'` doubled per the YAML spec).
///
/// The allowed plain-scalar charset is alphanumerics plus
/// `- _ . / * + = ~` (covers common branch patterns like
/// `release/*`, version tags, and workflow names). Anything else, or
/// any string that starts with a YAML indicator (`* & ? ! | > # @
/// \` `, `-`, `?`, `:`, `,`, `[`, `]`, `{`, `}`, `%`), gets
/// single-quoted to keep the output well-formed regardless of user
/// input.
fn yaml_scalar(s: &str) -> String {
    let is_plain_charset = !s.is_empty()
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '*' | '+' | '=' | '~')
        });
    let starts_with_indicator = matches!(
        s.chars().next(),
        Some('-') | Some('?') | Some(':') | Some(',') | Some('[') | Some(']') | Some('{')
            | Some('}') | Some('#') | Some('&') | Some('*') | Some('!') | Some('|') | Some('>')
            | Some('%') | Some('@') | Some('`')
    );
    if is_plain_charset && !starts_with_indicator {
        s.to_string()
    } else {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('\'');
        for ch in s.chars() {
            if ch == '\'' {
                out.push_str("''");
            } else {
                out.push(ch);
            }
        }
        out.push('\'');
        out
    }
}

/// Single-quote a value for inclusion in a POSIX `sh` / `bash` command.
///
/// Single-quoting in POSIX shell is literal — nothing inside is
/// interpreted, so the only escape needed is for embedded `'`, which
/// we replace with `'\''` (close-quote, escaped quote, re-open).
fn shell_arg(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
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
        assert!(yaml.contains("cargo clippy --all-targets --all-features"));
        assert!(yaml.contains("-D warnings"));
    }

    #[test]
    fn fmt_job_added_when_requested() {
        let yaml = Generator::new().with_fmt().generate();
        assert!(yaml.contains("fmt:"));
        assert!(yaml.contains("cargo fmt --all -- --check"));
    }

    #[test]
    fn docs_job_added_when_requested() {
        let yaml = Generator::new().with_docs().generate();
        assert!(yaml.contains("docs:"));
        assert!(yaml.contains("RUSTDOCFLAGS"));
        assert!(yaml.contains("cargo doc --all-features --no-deps"));
    }

    #[test]
    fn msrv_job_uses_pinned_toolchain() {
        let yaml = Generator::new().with_msrv("1.85").generate();
        assert!(yaml.contains("rust-toolchain@1.85"));
        assert!(yaml.contains("MSRV (Rust 1.85)"));
    }

    #[test]
    fn matrix_os_appears_in_test_job() {
        let yaml = Generator::new()
            .matrix_os(["ubuntu-latest", "macos-latest", "windows-latest"])
            .generate();
        assert!(yaml.contains("[ubuntu-latest, macos-latest, windows-latest]"));
        assert!(yaml.contains("runs-on: ${{ matrix.os }}"));
    }

    #[test]
    fn empty_matrix_falls_back_to_default() {
        let yaml: String = Generator::new().matrix_os(Vec::<&str>::new()).generate();
        assert!(yaml.contains("[ubuntu-latest]"));
    }

    #[test]
    fn branches_drive_both_push_and_pr_filters() {
        let yaml = Generator::new().branches(["main", "release/*"]).generate();
        let count = yaml.matches("branches: [main, release/*]").count();
        assert_eq!(count, 2); // once for push, once for pull_request
    }

    #[test]
    fn cache_action_present_by_default() {
        let yaml = Generator::new().generate();
        assert!(yaml.contains("Swatinem/rust-cache@v2"));
    }

    #[test]
    fn cache_action_removed_when_disabled() {
        let yaml = Generator::new().with_cache(false).generate();
        assert!(!yaml.contains("Swatinem/rust-cache"));
    }

    #[test]
    fn path_dep_clone_step_emitted() {
        let yaml = Generator::new()
            .with_path_dep(PathDep::new(
                "dev-report",
                "https://github.com/jamesgober/dev-report.git",
            ))
            .with_path_dep(PathDep::new(
                "dev-tools",
                "https://github.com/jamesgober/dev-tools.git",
            ))
            .generate();
        assert!(yaml.contains("Check out sibling crates (path deps)"));
        assert!(yaml.contains(
            "git clone --depth 1 'https://github.com/jamesgober/dev-report.git' '../dev-report'"
        ));
        assert!(yaml.contains(
            "git clone --depth 1 'https://github.com/jamesgober/dev-tools.git' '../dev-tools'"
        ));
    }

    #[test]
    fn yaml_scalar_quotes_workflow_name_with_colon() {
        let yaml = Generator::new()
            .workflow_name("Build: CI Pipeline")
            .generate();
        assert!(yaml.contains("name: 'Build: CI Pipeline'"));
    }

    #[test]
    fn yaml_scalar_doubles_embedded_single_quote() {
        let yaml = Generator::new().workflow_name("Don't break").generate();
        assert!(yaml.contains("name: 'Don''t break'"));
    }

    #[test]
    fn yaml_scalar_quotes_branch_with_comma() {
        let yaml = Generator::new().branches(["main", "release,foo"]).generate();
        assert!(yaml.contains("branches: [main, 'release,foo']"));
    }

    #[test]
    fn yaml_scalar_quotes_branch_starting_with_indicator() {
        let yaml = Generator::new().branches(["main", "*release"]).generate();
        assert!(yaml.contains("branches: [main, '*release']"));
    }

    #[test]
    fn shell_arg_escapes_embedded_single_quote() {
        let yaml = Generator::new()
            .with_path_dep(PathDep::new("weird-name", "https://x.com/o'malley.git"))
            .generate();
        // Single quote escape sequence: '\''  (close-quote, backslash, quote, re-open).
        assert!(yaml.contains("'https://x.com/o'\\''malley.git' '../weird-name'"));
    }

    #[test]
    fn no_default_features_build_emitted_when_requested() {
        let yaml = Generator::new().with_no_default_features_build().generate();
        assert!(yaml.contains("Build (no default features)"));
        assert!(yaml.contains("cargo build --no-default-features --verbose"));
    }

    #[test]
    fn all_features_build_and_test_emitted_when_requested() {
        let yaml = Generator::new().with_all_features_build().generate();
        assert!(yaml.contains("Build (all features)"));
        assert!(yaml.contains("Test (all features)"));
        assert!(yaml.contains("cargo build --all-features --verbose"));
        assert!(yaml.contains("cargo test --all-features --verbose"));
    }

    #[test]
    fn workspace_flag_propagates_to_cargo_calls() {
        let yaml = Generator::new().with_workspace().generate();
        assert!(yaml.contains("cargo build --workspace --verbose"));
        assert!(yaml.contains("cargo test --workspace --verbose"));
    }

    #[test]
    fn features_flag_propagates_when_set() {
        let yaml = Generator::new().features("foo,bar").generate();
        assert!(yaml.contains("cargo build --features foo,bar --verbose"));
        assert!(yaml.contains("cargo test --features foo,bar --verbose"));
    }

    #[test]
    fn features_flag_omitted_for_explicit_all_or_none() {
        let yaml = Generator::new()
            .features("foo")
            .with_all_features_build()
            .with_no_default_features_build()
            .generate();
        // The default test job still gets --features foo
        assert!(yaml.contains("cargo build --features foo --verbose"));
        // The --no-default-features step doesn't double up with --features
        assert!(yaml.contains("cargo build --no-default-features --verbose"));
        assert!(!yaml.contains("cargo build --no-default-features --features"));
    }

    #[test]
    fn workflow_name_appears_at_top() {
        let yaml = Generator::new().workflow_name("Pipeline").generate();
        assert!(yaml.starts_with("name: Pipeline\n"));
    }

    #[test]
    fn output_is_deterministic() {
        let g = Generator::new()
            .matrix_os(["ubuntu-latest", "macos-latest"])
            .with_clippy()
            .with_fmt()
            .with_docs()
            .with_msrv("1.85")
            .with_no_default_features_build()
            .with_all_features_build()
            .with_path_dep(PathDep::new(
                "dev-report",
                "https://example.com/dev-report.git",
            ));
        let a = g.generate();
        let b = g.generate();
        assert_eq!(a, b);
    }

    #[test]
    fn msrv_job_uses_pinned_toolchain_action_ref() {
        let yaml = Generator::new().with_msrv("1.85").generate();
        assert!(yaml.contains("dtolnay/rust-toolchain@1.85"));
    }

    #[test]
    fn full_kitchen_sink_yaml_round_trip() {
        // Sanity: confirm a "everything enabled" configuration produces
        // valid-looking YAML with each section.
        let yaml = Generator::new()
            .workflow_name("Full CI")
            .branches(["main", "develop"])
            .matrix_os(["ubuntu-latest", "macos-latest", "windows-latest"])
            .with_clippy()
            .with_fmt()
            .with_docs()
            .with_msrv("1.85")
            .with_no_default_features_build()
            .with_all_features_build()
            .with_workspace()
            .with_path_dep(PathDep::new(
                "dev-report",
                "https://example.com/dev-report.git",
            ))
            .generate();

        for needle in [
            "name: 'Full CI'",
            "actions/checkout@v5",
            "branches: [main, develop]",
            "[ubuntu-latest, macos-latest, windows-latest]",
            "clippy:",
            "fmt:",
            "docs:",
            "msrv:",
            "MSRV (Rust 1.85)",
            "Build (no default features)",
            "Build (all features)",
            "Test (all features)",
            "git clone --depth 1 'https://example.com/dev-report.git' '../dev-report'",
            "Swatinem/rust-cache@v2",
        ] {
            assert!(
                yaml.contains(needle),
                "missing: {needle}\n--- yaml ---\n{yaml}"
            );
        }
    }

    #[test]
    fn path_dep_accessors_round_trip() {
        let d = PathDep::new("foo", "https://example.com/foo.git");
        assert_eq!(d.name(), "foo");
        assert_eq!(d.repo_url(), "https://example.com/foo.git");
    }

    #[test]
    fn default_target_is_github_actions() {
        assert_eq!(Generator::new().target_kind(), Target::GitHubActions);
    }
}
