//! Minimal example: generate a basic GitHub Actions workflow.
//!
//! Run with: `cargo run --example basic`

use dev_ci::{Generator, Target};

fn main() {
    let yaml = Generator::new()
        .target(Target::GitHubActions)
        .with_clippy()
        .with_fmt()
        .with_msrv("1.85")
        .generate();

    println!("{yaml}");
}
