//! Exit codes and output of the `dev-ci` binary.
#![cfg(feature = "cli")]

use std::path::PathBuf;
use std::process::{Command, Output};

fn dev_ci(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dev-ci"))
        .args(args)
        .output()
        .expect("spawn dev-ci")
}

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dev-ci-cli-test-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn print_writes_yaml_to_stdout_and_exits_zero() {
    let out = dev_ci(&[
        "generate",
        "--print",
        "--with",
        "clippy,msrv",
        "--msrv",
        "1.75",
        "--branches",
        "main,,dev",
        "--matrix",
        " ubuntu-latest , windows-latest",
    ]);
    assert_eq!(out.status.code(), Some(0));
    let yaml = String::from_utf8(out.stdout).unwrap();
    assert!(yaml.starts_with("name: CI\n"));
    assert!(yaml.contains("branches: [main, dev]"), "{yaml}");
    assert!(
        yaml.contains("os: [ubuntu-latest, windows-latest]"),
        "{yaml}"
    );
    assert!(yaml.contains("dtolnay/rust-toolchain@1.75"));
}

#[test]
fn argument_parse_errors_exit_two() {
    assert_eq!(dev_ci(&["generate", "--bogus"]).status.code(), Some(2));
    assert_eq!(
        dev_ci(&["generate", "--print", "--output", "x.yml"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        dev_ci(&["generate", "--target", "gitlab"]).status.code(),
        Some(2)
    );
}

#[test]
fn semantic_errors_exit_one_with_message() {
    let out = dev_ci(&["generate", "--print", "--with", "msrv"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--with msrv requires --msrv"));

    let out = dev_ci(&["generate", "--print", "--with", "nope"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown job"));

    for bad in ["", "1.85 #x"] {
        let out = dev_ci(&["generate", "--print", "--msrv", bad]);
        assert_eq!(out.status.code(), Some(1), "--msrv {bad:?}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("--msrv must be"));
    }

    let out = dev_ci(&["generate", "--print", "--path-dep", "no-equals"]);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn writes_file_and_creates_parent_dirs() {
    let dir = scratch_dir("write");
    let target = dir.join("nested").join("workflows").join("ci.yml");
    let out = dev_ci(&["generate", "--output", target.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let yaml = std::fs::read_to_string(&target).unwrap();
    assert!(yaml.contains("jobs:"));
    assert!(!yaml.contains('\r'), "output uses LF line endings");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn io_error_exits_one() {
    let dir = scratch_dir("ioerr");
    // A regular file where a parent directory is needed.
    let blocker = dir.join("blocker");
    std::fs::write(&blocker, "x").unwrap();
    let target = blocker.join("ci.yml");
    let out = dev_ci(&["generate", "--output", target.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&out.stderr).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
