//! Generate a small but complete GitHub Actions workflow.
//!
//! ```text
//! cargo run --example basic
//! ```
//!
//! Output is what most pure-Rust crates need: `test` (single OS) plus
//! the standard quality gates (`clippy`, `fmt`, `docs`, `msrv`).

use dev_ci::{Generator, Target};

fn main() {
    let yaml = Generator::new()
        .target(Target::GitHubActions)
        .with_clippy()
        .with_fmt()
        .with_docs()
        .with_msrv("1.85")
        .generate();

    println!("{yaml}");
}
