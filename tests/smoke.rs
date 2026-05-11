use dev_ci::{Generator, Target};

#[test]
fn smoke_default_target_is_github_actions() {
    let g = Generator::new();
    assert_eq!(g.target_kind(), Target::GitHubActions);
}

#[test]
fn smoke_default_yaml_contains_test_job() {
    let yaml = Generator::new().generate();
    assert!(yaml.contains("jobs:"));
    assert!(yaml.contains("test:"));
}

#[test]
fn smoke_uses_node_24_compatible_checkout() {
    let yaml = Generator::new().generate();
    // actions/checkout@v5 is the Node 24 release. Earlier versions
    // emit deprecation warnings on GitHub-hosted runners.
    assert!(yaml.contains("actions/checkout@v5"));
    assert!(!yaml.contains("actions/checkout@v4"));
    assert!(!yaml.contains("actions/checkout@v3"));
}

#[test]
fn smoke_clippy_job_optional() {
    let with = Generator::new().with_clippy().generate();
    let without = Generator::new().generate();
    assert!(with.contains("clippy:"));
    assert!(!without.contains("clippy:"));
}

#[test]
fn smoke_fmt_job_optional() {
    let with = Generator::new().with_fmt().generate();
    let without = Generator::new().generate();
    assert!(with.contains("fmt:"));
    assert!(!without.contains("fmt:"));
}

#[test]
fn smoke_msrv_pins_toolchain() {
    let yaml = Generator::new().with_msrv("1.85").generate();
    assert!(yaml.contains("rust-toolchain@1.85"));
}

#[test]
fn smoke_determinism_same_config_same_yaml() {
    let a = Generator::new().with_clippy().with_fmt().generate();
    let b = Generator::new().with_clippy().with_fmt().generate();
    assert_eq!(a, b, "same configuration must produce byte-identical YAML");
}
