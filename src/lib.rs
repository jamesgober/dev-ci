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

/// Version of this crate as compiled, taken from its `Cargo.toml`.
///
/// Lets tools that bundle this crate, such as the `dev` CLI in
/// `dev-tools`, report the version that is actually linked.
///
/// # Example
///
/// ```
/// assert!(!dev_ci::VERSION.is_empty());
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    ///
    /// The clone commands are quoted for a POSIX shell and the step sets
    /// `shell: bash`, so it behaves the same on Windows runners.
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
    ///
    /// `version` becomes the `dtolnay/rust-toolchain@<version>` ref, so it
    /// should be a toolchain name such as `1.75` or `1.75.0`. Unusual
    /// values are YAML-quoted rather than rejected; the output stays
    /// well-formed but the action will not resolve them.
    ///
    /// When the repository has no committed `Cargo.lock`, the job first
    /// runs `cargo generate-lockfile` on stable with
    /// `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback`, so the
    /// dependency versions respect the `rust-version` in `Cargo.toml`.
    /// Without that step the old toolchain would pick the newest release
    /// of every dependency and could fail on one that needs a newer Rust.
    /// Set `rust-version` to the same value as `version` for this to
    /// have an effect. A committed `Cargo.lock` is used as is.
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
            out.push_str(&yaml_scalar(os));
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
        let ws = self.cargo_flags_string(false, false);
        out.push_str("      - name: Clippy (all features)\n");
        writeln!(
            out,
            "        run: cargo clippy{ws} --all-targets --all-features -- -D warnings"
        )
        .unwrap();
        out.push_str("      - name: Clippy (no default features)\n");
        writeln!(
            out,
            "        run: cargo clippy{ws} --all-targets --no-default-features -- -D warnings"
        )
        .unwrap();
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
        let ws = self.cargo_flags_string(false, false);
        writeln!(out, "      - run: cargo doc{ws} --all-features --no-deps").unwrap();
    }

    /// MSRV job.
    ///
    /// Without a committed `Cargo.lock`, an old toolchain resolves the
    /// newest version of every dependency, which may need a newer Rust
    /// (or a newer Cargo just to read its manifest) and fail the job even
    /// though the crate itself is fine. The job therefore first resolves
    /// a lockfile with stable Cargo's MSRV-aware resolver
    /// (`CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback`, which reads
    /// `rust-version` from `Cargo.toml`), and only then switches to the
    /// pinned toolchain. A committed `Cargo.lock` is left untouched.
    fn write_msrv_job(&self, out: &mut String, msrv: &str) {
        writeln!(out, "\n  msrv:").unwrap();
        writeln!(
            out,
            "    name: {}",
            yaml_block_scalar(&format!("MSRV (Rust {msrv})"))
        )
        .unwrap();
        out.push_str("    runs-on: ubuntu-latest\n");
        out.push_str("    steps:\n");
        self.write_checkout_and_path_deps(out);
        out.push_str("      - uses: dtolnay/rust-toolchain@stable\n");
        out.push_str("      - name: Resolve dependencies compatible with rust-version\n");
        out.push_str("        shell: bash\n");
        out.push_str("        run: if [ ! -f Cargo.lock ]; then cargo generate-lockfile; fi\n");
        out.push_str("        env:\n");
        out.push_str("          CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS: fallback\n");
        self.write_toolchain_and_cache(out, None, Some(msrv));
        let extras = self.cargo_flags_string(true, false);
        writeln!(
            out,
            "      - run: {}",
            yaml_block_scalar(&format!("cargo build{extras}"))
        )
        .unwrap();
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
        self.write_checkout_and_path_deps(out);
        self.write_toolchain_and_cache(out, component, toolchain_pin);
    }

    fn write_checkout_and_path_deps(&self, out: &mut String) {
        out.push_str("      - uses: actions/checkout@v5\n");
        if !self.path_deps.is_empty() {
            out.push_str("      - name: Check out sibling crates (path deps)\n");
            // The arguments are quoted for a POSIX shell. Pin the shell so
            // the step means the same thing on Windows runners, whose
            // default shell is PowerShell.
            out.push_str("        shell: bash\n");
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
    }

    fn write_toolchain_and_cache(
        &self,
        out: &mut String,
        component: Option<&str>,
        toolchain_pin: Option<&str>,
    ) {
        let toolchain = toolchain_pin.unwrap_or("stable");
        writeln!(
            out,
            "      - uses: {}",
            yaml_block_scalar(&format!("dtolnay/rust-toolchain@{toolchain}"))
        )
        .unwrap();
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
        let command = format!("cargo {cmd}{flags}{extra} --verbose");
        writeln!(out, "      - name: {name}").unwrap();
        if command.contains('\'') {
            // The feature list was shell-quoted (POSIX rules). Run it under
            // bash on every OS, including Windows runners.
            out.push_str("        shell: bash\n");
        }
        writeln!(out, "        run: {}", yaml_block_scalar(&command)).unwrap();
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
                    // Plain feature lists (`foo,bar`, `serde/std`) stay
                    // unquoted; anything else is quoted for the shell so
                    // spaces or metacharacters cannot split or extend the
                    // command.
                    if f.chars().all(|c| {
                        c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '+' | '.' | ',' | '/')
                    }) {
                        s.push_str(f);
                    } else {
                        s.push_str(&shell_arg(f));
                    }
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
///
/// Strings that a YAML parser would read as something other than a
/// string (`true`, `no`, `null`, `~`, `1.10`, `0x1F`, ...) are quoted too,
/// so a branch named `1.10` is not turned into the number `1.1`. Strings
/// with control characters (such as a newline) are emitted double-quoted
/// with escapes, because a single-quoted scalar would fold the newline.
fn yaml_scalar(s: &str) -> String {
    if s.chars().any(|c| c.is_control() && c != '\t') {
        return yaml_double_quoted(s);
    }
    let is_plain_charset = !s.is_empty()
        && !yaml_non_string_plain(s)
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '*' | '+' | '=' | '~')
        });
    let starts_with_indicator = matches!(
        s.chars().next(),
        Some('-')
            | Some('?')
            | Some(':')
            | Some(',')
            | Some('[')
            | Some(']')
            | Some('{')
            | Some('}')
            | Some('#')
            | Some('&')
            | Some('*')
            | Some('!')
            | Some('|')
            | Some('>')
            | Some('%')
            | Some('@')
            | Some('`')
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

/// `true` when YAML would resolve the plain scalar `s` to a bool, null,
/// or number instead of a string (covering both YAML 1.1 and 1.2 forms).
/// Anything that starts with a digit (after an optional sign or a leading
/// `.`) counts as numeric; quoting a string that is not really a number
/// is harmless.
fn yaml_non_string_plain(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "true"
            | "false"
            | "yes"
            | "no"
            | "on"
            | "off"
            | "y"
            | "n"
            | "null"
            | "~"
            | ".inf"
            | "+.inf"
            | "-.inf"
            | ".nan"
    ) {
        return true;
    }
    let t = s.trim_start_matches(['+', '-']);
    let t = t.strip_prefix('.').unwrap_or(t);
    t.starts_with(|c: char| c.is_ascii_digit())
}

/// YAML double-quoted scalar with escapes for `\`, `"` and control
/// characters.
fn yaml_double_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                write!(out, "\\u{:04X}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Emit `s` as the value of a block mapping entry (`name: ...`,
/// `run: ...`, `uses: ...`).
///
/// Block context allows more in a plain scalar than flow context
/// (spaces, parentheses, quotes after the first character), so ordinary
/// values such as `cargo build --verbose` or `MSRV (Rust 1.85)` stay
/// unquoted. A value is quoted when it is empty, has surrounding
/// whitespace, starts with a YAML indicator, contains `: ` or ` #`,
/// ends with `:`, or would be read as a bool, null, or number. Control
/// characters force a double-quoted scalar.
fn yaml_block_scalar(s: &str) -> String {
    if s.chars().any(|c| c.is_control()) {
        return yaml_double_quoted(s);
    }
    let needs_quotes = s.is_empty()
        || s.trim() != s
        || s.starts_with([
            '-', '?', ':', ',', '[', ']', '{', '}', '#', '&', '*', '!', '|', '>', '\'', '"', '%',
            '@', '`',
        ])
        || s.contains(": ")
        || s.contains(" #")
        || s.ends_with(':')
        || yaml_non_string_plain(s);
    if !needs_quotes {
        return s.to_string();
    }
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
        let yaml = Generator::new()
            .branches(["main", "release,foo"])
            .generate();
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
    fn matrix_entries_are_yaml_quoted_when_needed() {
        let yaml = Generator::new()
            .matrix_os(["ubuntu-latest", "self-hosted, linux", "on"])
            .generate();
        assert!(
            yaml.contains("os: [ubuntu-latest, 'self-hosted, linux', 'on']"),
            "{yaml}"
        );
    }

    #[test]
    fn yaml_scalar_quotes_values_yaml_would_not_read_as_strings() {
        for s in [
            "1.10", "2024", "-1", ".5", "true", "No", "off", "null", "~", "y",
        ] {
            assert_eq!(yaml_scalar(s), format!("'{s}'"), "{s}");
        }
        for s in ["main", "release/*", "v1.2", "ubuntu-22.04", "feature-x"] {
            assert_eq!(yaml_scalar(s), s, "{s}");
        }
        let yaml = Generator::new().branches(["1.10"]).generate();
        assert!(yaml.contains("branches: ['1.10']"));
    }

    #[test]
    fn yaml_scalar_double_quotes_control_characters() {
        assert_eq!(yaml_scalar("a\nb"), "\"a\\nb\"");
        assert_eq!(yaml_scalar("q\"\\\u{1}"), "\"q\\\"\\\\\\u0001\"");
        let yaml = Generator::new().workflow_name("CI\nevil: 1").generate();
        assert!(yaml.starts_with("name: \"CI\\nevil: 1\"\n"), "{yaml}");
    }

    #[test]
    fn yaml_block_scalar_keeps_plain_commands_plain() {
        for s in [
            "cargo build --verbose",
            "MSRV (Rust 1.85)",
            "dtolnay/rust-toolchain@1.85",
            "cargo build --features 'a b' --verbose",
        ] {
            assert_eq!(yaml_block_scalar(s), s);
        }
        assert_eq!(yaml_block_scalar("x: y"), "'x: y'");
        assert_eq!(yaml_block_scalar("a #b"), "'a #b'");
        assert_eq!(yaml_block_scalar("ends:"), "'ends:'");
        assert_eq!(yaml_block_scalar("'q"), "'''q'");
        assert_eq!(yaml_block_scalar(""), "''");
        assert_eq!(yaml_block_scalar("1.85"), "'1.85'");
        assert_eq!(yaml_block_scalar("a\nb"), "\"a\\nb\"");
    }

    #[test]
    fn msrv_value_cannot_break_out_of_the_job() {
        let yaml = Generator::new().with_msrv("1.85 #x: y").generate();
        assert!(
            yaml.contains("    name: 'MSRV (Rust 1.85 #x: y)'\n"),
            "{yaml}"
        );
        assert!(
            yaml.contains("      - uses: 'dtolnay/rust-toolchain@1.85 #x: y'\n"),
            "{yaml}"
        );
        let yaml = Generator::new()
            .with_msrv("1.85\nmalicious: true")
            .generate();
        assert!(!yaml.contains("\nmalicious: true"), "{yaml}");
    }

    #[test]
    fn msrv_job_resolves_lockfile_before_switching_toolchain() {
        let yaml = Generator::new().with_msrv("1.75").generate();
        let msrv = &yaml[yaml.find("  msrv:").unwrap()..];
        let stable = msrv.find("dtolnay/rust-toolchain@stable").unwrap();
        let resolve = msrv.find("cargo generate-lockfile").unwrap();
        let pinned = msrv.find("dtolnay/rust-toolchain@1.75").unwrap();
        let build = msrv.find("run: cargo build").unwrap();
        assert!(
            stable < resolve && resolve < pinned && pinned < build,
            "{msrv}"
        );
        assert!(msrv.contains("if [ ! -f Cargo.lock ]; then cargo generate-lockfile; fi"));
        assert!(msrv.contains("CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS: fallback"));
    }

    #[test]
    fn features_with_shell_metacharacters_are_quoted() {
        let yaml = Generator::new().features("a b; rm -rf /").generate();
        assert!(
            yaml.contains("run: cargo build --features 'a b; rm -rf /' --verbose"),
            "{yaml}"
        );
        assert!(yaml.contains("shell: bash"));
        let yaml = Generator::new().features("x #y").generate();
        assert!(
            yaml.contains("run: 'cargo build --features ''x #y'' --verbose'"),
            "{yaml}"
        );
        // Plain lists are untouched and need no shell override.
        let yaml = Generator::new().features("serde/std,derive").generate();
        assert!(yaml.contains("run: cargo build --features serde/std,derive --verbose"));
        assert!(!yaml.contains("shell: bash"));
    }

    #[test]
    fn path_dep_clone_step_runs_under_bash() {
        let yaml = Generator::new()
            .matrix_os(["windows-latest"])
            .with_path_dep(PathDep::new("dev-report", "https://example.com/r.git"))
            .generate();
        assert!(yaml.contains(
            "      - name: Check out sibling crates (path deps)\n        shell: bash\n        run: |\n"
        ));
    }

    #[test]
    fn workspace_flag_reaches_clippy_and_docs() {
        let yaml = Generator::new()
            .with_workspace()
            .with_clippy()
            .with_docs()
            .generate();
        assert!(
            yaml.contains("cargo clippy --workspace --all-targets --all-features -- -D warnings")
        );
        assert!(yaml.contains(
            "cargo clippy --workspace --all-targets --no-default-features -- -D warnings"
        ));
        assert!(yaml.contains("cargo doc --workspace --all-features --no-deps"));
        // Without the flag the commands are unchanged.
        let yaml = Generator::new().with_clippy().with_docs().generate();
        assert!(yaml.contains("cargo clippy --all-targets --all-features -- -D warnings"));
        assert!(yaml.contains("cargo doc --all-features --no-deps"));
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
