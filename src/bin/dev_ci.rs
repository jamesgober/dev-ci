//! `dev-ci` CLI binary.
//!
//! Usage:
//!
//! ```text
//! dev-ci generate \
//!     --target github-actions \
//!     --output .github/workflows/ci.yml \
//!     --with clippy,fmt,docs,msrv \
//!     --msrv 1.85 \
//!     --matrix ubuntu-latest,macos-latest,windows-latest \
//!     --path-dep dev-report=https://github.com/jamesgober/dev-report.git
//! ```

use std::fs;
use std::io::{self, Write as _};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use dev_ci::{Generator, PathDep, Target};

#[derive(Debug, Parser)]
#[command(
    name = "dev-ci",
    version,
    about = "Generate calibrated CI pipelines for Rust projects.",
    long_about = "Generate calibrated CI pipelines tailored to the dev-* features a project uses. Default output is `.github/workflows/ci.yml`."
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Generate a workflow file.
    Generate(GenerateArgs),
}

#[derive(Debug, Parser)]
struct GenerateArgs {
    /// Target CI platform.
    #[arg(long, value_enum, default_value_t = TargetArg::GithubActions)]
    target: TargetArg,

    /// Where to write the workflow. Defaults to the standard path for
    /// the chosen target. Pass `-` to write to stdout.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Write to stdout instead of a file.
    #[arg(long, conflicts_with = "output")]
    print: bool,

    /// Workflow `name:` field.
    #[arg(long, default_value = "CI")]
    workflow_name: String,

    /// Comma-separated branch list for both `push` and `pull_request`.
    #[arg(long, default_value = "main", value_delimiter = ',')]
    branches: Vec<String>,

    /// Comma-separated OS runner list for the test matrix.
    #[arg(long, default_value = "ubuntu-latest", value_delimiter = ',')]
    matrix: Vec<String>,

    /// Comma-separated job names to include in addition to `test`.
    /// Supported: `clippy`, `fmt`, `docs`, `msrv`.
    #[arg(long, value_delimiter = ',')]
    with: Vec<String>,

    /// MSRV version (e.g. `1.85`) for the MSRV job. Required when
    /// `--with msrv` is set; ignored otherwise.
    #[arg(long)]
    msrv: Option<String>,

    /// Comma-separated cargo feature list for the test job.
    #[arg(long)]
    features: Option<String>,

    /// Add a `--no-default-features` build step under the `test` job.
    #[arg(long)]
    no_default_features_build: bool,

    /// Add `--all-features` build + test steps under the `test` job.
    #[arg(long)]
    all_features_build: bool,

    /// Pass `--workspace` to every cargo invocation.
    #[arg(long)]
    workspace: bool,

    /// Disable the `Swatinem/rust-cache@v2` action.
    #[arg(long)]
    no_cache: bool,

    /// Sibling path-dep declaration in `name=repo-url` form. Repeatable.
    ///
    /// Example: `--path-dep dev-report=https://github.com/jamesgober/dev-report.git`
    #[arg(long = "path-dep", value_name = "NAME=URL")]
    path_deps: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TargetArg {
    #[value(name = "github-actions")]
    GithubActions,
}

impl TargetArg {
    fn to_lib(self) -> Target {
        match self {
            Self::GithubActions => Target::GitHubActions,
        }
    }

    fn default_output_path(self) -> PathBuf {
        match self {
            Self::GithubActions => PathBuf::from(".github/workflows/ci.yml"),
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let res = match cli.command {
        Cmd::Generate(args) => run_generate(args),
    };
    match res {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("dev-ci: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_generate(args: GenerateArgs) -> Result<(), String> {
    let mut gen = Generator::new()
        .target(args.target.to_lib())
        .workflow_name(args.workflow_name)
        .branches(args.branches)
        .matrix_os(args.matrix);

    if args.no_cache {
        gen = gen.with_cache(false);
    }
    if args.workspace {
        gen = gen.with_workspace();
    }
    if let Some(f) = args.features {
        gen = gen.features(f);
    }
    if args.no_default_features_build {
        gen = gen.with_no_default_features_build();
    }
    if args.all_features_build {
        gen = gen.with_all_features_build();
    }

    for raw in &args.path_deps {
        let (name, url) = parse_path_dep(raw)?;
        gen = gen.with_path_dep(PathDep::new(name, url));
    }

    for job in &args.with {
        match job.trim().to_ascii_lowercase().as_str() {
            "" => {}
            "clippy" => gen = gen.with_clippy(),
            "fmt" => gen = gen.with_fmt(),
            "docs" => gen = gen.with_docs(),
            "msrv" => {
                let v = args
                    .msrv
                    .as_deref()
                    .ok_or_else(|| "--with msrv requires --msrv <VERSION>".to_string())?;
                gen = gen.with_msrv(v);
            }
            other => return Err(format!("unknown job in --with: {other:?}")),
        }
    }
    // If --msrv was supplied without an explicit --with msrv, honor it too.
    if !args.with.iter().any(|j| j.eq_ignore_ascii_case("msrv")) {
        if let Some(v) = &args.msrv {
            gen = gen.with_msrv(v.clone());
        }
    }

    let yaml = gen.generate();

    let stdout_mode =
        args.print || matches!(args.output.as_ref().and_then(|p| p.to_str()), Some("-"));

    if stdout_mode {
        io::stdout()
            .write_all(yaml.as_bytes())
            .map_err(|e| format!("failed to write to stdout: {e}"))?;
        return Ok(());
    }

    let target_path = args
        .output
        .unwrap_or_else(|| args.target.default_output_path());
    if let Some(parent) = target_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("create_dir_all({}): {e}", parent.display()))?;
        }
    }
    fs::write(&target_path, yaml).map_err(|e| format!("write {}: {e}", target_path.display()))?;
    eprintln!("wrote {}", target_path.display());
    Ok(())
}

fn parse_path_dep(raw: &str) -> Result<(&str, &str), String> {
    let (name, url) = raw
        .split_once('=')
        .ok_or_else(|| format!("--path-dep must be name=url; got {raw:?}"))?;
    if name.is_empty() || url.is_empty() {
        return Err(format!(
            "--path-dep name and url must be non-empty; got {raw:?}"
        ));
    }
    Ok((name, url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_path_dep_splits_on_first_equals() {
        let (name, url) = parse_path_dep("foo=https://example.com/foo.git").unwrap();
        assert_eq!(name, "foo");
        assert_eq!(url, "https://example.com/foo.git");
    }

    #[test]
    fn parse_path_dep_rejects_missing_equals() {
        assert!(parse_path_dep("foo").is_err());
    }

    #[test]
    fn parse_path_dep_rejects_empty_name_or_url() {
        assert!(parse_path_dep("=url").is_err());
        assert!(parse_path_dep("name=").is_err());
    }
}
