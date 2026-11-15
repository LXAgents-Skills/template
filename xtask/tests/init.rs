//! `init`, end to end, on a copy of the repository: it renames everything, strips the template's
//! own text, builds the renamed server, and lists its tools under the new prefix.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Copy the tracked shape of the repository: everything but build output, clones and `.git`.
fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap().flatten() {
        let name = entry.file_name();
        if matches!(
            name.to_str(),
            Some("target" | ".git" | "mcps" | "node_modules")
        ) {
            continue;
        }
        let (source, target) = (entry.path(), to.join(&name));
        if source.is_dir() {
            copy(&source, &target);
        } else {
            fs::copy(&source, &target).unwrap();
        }
    }
}

/// Every text file in `dir` that still contains `needle`.
fn still_naming(dir: &Path, needle: &str, found: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            // `target/` is build output; `xtask/` is init itself, which names the markers it strips
            // and, in this test, the old identity on purpose.
            if !path.ends_with("target") && !path.ends_with("xtask") {
                still_naming(&path, needle, found);
            }
        } else if fs::read_to_string(&path).is_ok_and(|text| text.contains(needle)) {
            found.push(path.display().to_string());
        }
    }
}

#[test]
fn init_renames_a_copy_and_proves_its_tools() {
    let copy_root = std::env::temp_dir().join(format!("xtask-init-{}", std::process::id()));
    let _ = fs::remove_dir_all(&copy_root);
    copy(&root(), &copy_root);

    // A shared target directory, so the copy reuses every dependency already built.
    let target = root().join("target").join("init-e2e");
    let output = Command::new(env!("CARGO"))
        .args([
            "run",
            "--quiet",
            "--package",
            "xtask",
            "--",
            "init",
            "--org",
            "Acme",
            "--repo",
            "my-skills",
        ])
        .current_dir(&copy_root)
        .env("CARGO_TARGET_DIR", &target)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        output.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with("acme_my_skills_automation ")),
        "{stdout}"
    );

    let identity = fs::read_to_string(copy_root.join("src/identity.rs")).unwrap();
    assert!(identity.contains("pub const ORG_NAME: &str = \"Acme\";"));
    assert!(identity.contains("pub const SERVER_ID: &str = \"acme-my-skills\";"));
    let manifest = fs::read_to_string(copy_root.join("Cargo.toml")).unwrap();
    assert!(
        manifest.contains("name = \"acme-my-skills\"") && manifest.contains("version = \"0.0.0\"")
    );
    assert!(!manifest.contains("\"template\""));

    for gone in [
        "tests/template_version.rs",
        ".agents/rules/template-version.md",
        "wiki/logs",
        ".agents/memory/tasks",
    ] {
        assert!(!copy_root.join(gone).exists(), "{gone} survived init");
    }
    for needle in [
        "lxagents_skills_template_",
        "LXAgents-Skills/template",
        "lxagents-template",
        "<!-- template-only -->",
    ] {
        let mut found = Vec::new();
        still_naming(&copy_root, needle, &mut found);
        assert!(found.is_empty(), "{needle} survived init in {found:?}");
    }

    fs::remove_dir_all(&copy_root).unwrap();
}
