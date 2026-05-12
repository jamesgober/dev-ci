//! Generate a workflow for a crate with sibling path-deps — the
//! pattern the dev-* suite uses for its own CI (each crate clones
//! its siblings into `..` before running cargo).
//!
//! ```text
//! cargo run --example path_deps
//! ```

use dev_ci::{Generator, PathDep, Target};

fn main() {
    let yaml = Generator::new()
        .target(Target::GitHubActions)
        .matrix_os(["ubuntu-latest", "macos-latest", "windows-latest"])
        .with_path_dep(PathDep::new(
            "dev-report",
            "https://github.com/jamesgober/dev-report.git",
        ))
        .with_path_dep(PathDep::new(
            "dev-tools",
            "https://github.com/jamesgober/dev-tools.git",
        ))
        .with_clippy()
        .with_fmt()
        .with_docs()
        .with_msrv("1.85")
        .generate();

    println!("{yaml}");
}
