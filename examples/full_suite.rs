//! Generate a workflow with the full standard surface: 3-OS matrix,
//! every standard job, no-default + all-features build coverage,
//! workspace flag.
//!
//! ```text
//! cargo run --example full_suite
//! ```

use dev_ci::{Generator, Target};

fn main() {
    let yaml = Generator::new()
        .target(Target::GitHubActions)
        .workflow_name("CI")
        .branches(["main"])
        .matrix_os(["ubuntu-latest", "macos-latest", "windows-latest"])
        .with_workspace()
        .with_no_default_features_build()
        .with_all_features_build()
        .with_clippy()
        .with_fmt()
        .with_docs()
        .with_msrv("1.85")
        .generate();

    println!("{yaml}");
}
