//! The template is `0.0.0`, for good. See `.agents/rules/template-version.md`.

mod common;

use std::fs;
use std::process::Command;

#[test]
fn every_version_carrier_is_0_0_0() {
    assert_eq!(skill_server::version::VERSION, "0.0.0");

    let manifest = fs::read_to_string(common::root().join("Cargo.toml")).unwrap();
    assert!(
        manifest.contains("\nversion = \"0.0.0\"\n"),
        "Cargo.toml must stay at 0.0.0"
    );

    let lock = fs::read_to_string(common::root().join("Cargo.lock")).unwrap();
    let entry = format!("name = \"{}\"\nversion = \"0.0.0\"", env!("CARGO_PKG_NAME"));
    assert!(
        lock.contains(&entry),
        "Cargo.lock must carry {} 0.0.0",
        env!("CARGO_PKG_NAME")
    );
}

#[test]
fn there_are_no_tags() {
    let output = Command::new("git")
        .args(["tag", "--list"])
        .current_dir(common::root())
        .output();
    if let Some(output) = output.ok().filter(|output| output.status.success()) {
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "",
            "the template carries no tags"
        );
    }
}
