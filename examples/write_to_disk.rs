//! Generate a workflow and write it to `.github/workflows/ci.yml`.
//!
//! ```text
//! cargo run --example write_to_disk
//! ```
//!
//! Writes into a temporary directory by default so running the example
//! doesn't overwrite the host repo's CI file. Set
//! `DEV_CI_WRITE_TARGET=.github/workflows/ci.yml` to write into the
//! current crate instead.

use std::path::PathBuf;

use dev_ci::Generator;

fn main() {
    let yaml = Generator::new()
        .with_clippy()
        .with_fmt()
        .with_docs()
        .with_msrv("1.85")
        .generate();

    let target = std::env::var("DEV_CI_WRITE_TARGET")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("dev-ci-example/ci.yml"));

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).expect("create parent dir");
    }
    std::fs::write(&target, yaml).expect("write yaml");
    println!("wrote {}", target.display());
}
