//! Repository tasks, run as `cargo xtask <task>`.
//!
//! ```text
//! cargo xtask init -- --org LXAgents-Skills --repo my-skills
//! ```
//!
//! `init` turns a fresh copy of the template into a new repository. It rewrites the identity in
//! `src/identity.rs` and `Cargo.toml`, swaps the old tool prefix, `org/repo` slug and server ID for
//! the new ones in every markdown file, the manifests, the tests and the Dockerfile, drops every
//! template-only block and file, clears the template's own history (task records, decisions,
//! release logs), then builds the renamed server and lists its tools to prove every ID is still
//! valid. Run it once, on a clean working tree, and review `git diff` before committing.

mod args;
mod identity;
mod rewrite;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use args::{Flags, parse_args};
use identity::{Identity, identity_for};
use rewrite::{replace_all, rewrite_identity, rewrite_manifest, strip_template_only};
use skill_server::identity::{ORG_NAME, REPO_NAME, SERVER_ID, TOOL_PREFIX, id_part};

const HELP: &str = "Usage: cargo xtask init -- --org <GitHub org> --repo <repository> [options]

Options
  --server-id <id>   server name; default: first word of the org, a dash, the repo
  --title <text>     human-readable title; default: \"<Org word> <Repo In Title Case>\"
  --summary <text>   one sentence on what the set is
  --keep-history     keep the template's task records, decisions and release logs
  --dry-run          print what would change and write nothing
  --force            run even though origin is the template repository itself
  --help             show this help";

/// Paths the template's own history lives in. A new repository starts without them.
const HISTORY: &[&str] = &[
    ".agents/memory/tasks",
    ".agents/memory/decisions",
    ".agents/memory/state",
    "wiki/logs",
];

/// Files that are true only of the template itself, including its version pin.
const TEMPLATE_ONLY: &[&str] = &[
    "wiki/guides/create-a-new-repository.md",
    ".agents/rules/template-version.md",
    "tests/template_version.rs",
];

/// Files outside markdown that name the server, the slug or the prefix.
const NAMED: &[&str] = &["Cargo.toml", "xtask/Cargo.toml", "Dockerfile"];

/// Fresh indexes for the history that was cleared.
const FRESH_INDEXES: &[(&str, &str)] = &[
    (
        ".agents/index/memory-index.md",
        "---\nname: memory-index\ndescription: Router over .agents/memory/ - one row per task, decision, or state file, read every session so work continues instead of restarting.\n---\n\n# Memory Index\n\n| File | Scope |\n|---|---|\n",
    ),
    (
        ".agents/index/logs-index.md",
        "---\nname: logs-index\ndescription: Router over wiki/logs/ - one row per release, newest first.\n---\n\n# Logs Index\n\n| Version | Changelog |\n|---|---|\n",
    ),
];

/// The repository root: the parent of this crate.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the repository")
        .to_path_buf()
}

/// Every file under `dir` whose name passes `keep`, skipping build output, clones and `.git`.
fn walk(dir: &Path, keep: &dyn Fn(&str) -> bool, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if matches!(name.as_str(), "target" | ".git" | "mcps" | "node_modules") {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            walk(&path, keep, found);
        } else if keep(&name) {
            found.push(path);
        }
    }
}

/// Read, transform, and write back if changed. Returns the relative path when it changed.
fn edit(
    root: &Path,
    path: &Path,
    dry_run: bool,
    transform: impl Fn(&str) -> Result<String, String>,
) -> Result<Option<String>, String> {
    let before =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let after = transform(&before)?;
    if after == before {
        return Ok(None);
    }
    if !dry_run {
        fs::write(path, after).map_err(|error| format!("{}: {error}", path.display()))?;
    }
    Ok(Some(
        path.strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string(),
    ))
}

/// Whether this checkout's origin is the template repository itself.
fn is_template_itself(root: &Path) -> bool {
    let Ok(output) = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output()
    else {
        return false;
    };
    let slug = format!("{ORG_NAME}/{REPO_NAME}").to_lowercase();
    let url = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_lowercase();
    output.status.success() && url.trim_end_matches(".git").ends_with(&slug)
}

/// The replacements, in order: the tool prefix, the slug, the server ID.
fn pairs(next: &Identity) -> Vec<(String, String)> {
    vec![
        (
            format!("{}_", *TOOL_PREFIX),
            format!("{}_{}_", id_part(&next.org), id_part(&next.repo)),
        ),
        (
            format!("{ORG_NAME}/{REPO_NAME}"),
            format!("{}/{}", next.org, next.repo),
        ),
        (SERVER_ID.to_string(), next.server_id.clone()),
    ]
}

/// Rename the copy. Returns the report lines.
fn rename(root: &Path, flags: &Flags, next: &Identity) -> Result<Vec<String>, String> {
    let dry_run = flags.dry_run;
    let pairs = pairs(next);
    let mut changed: Vec<String> = Vec::new();
    let mut removed: Vec<String> = Vec::new();

    let mut gone = |relative: &str| -> Result<(), String> {
        let path = root.join(relative);
        if path.exists() {
            if !dry_run {
                let result = if path.is_dir() {
                    fs::remove_dir_all(&path)
                } else {
                    fs::remove_file(&path)
                };
                result.map_err(|error| format!("{relative}: {error}"))?;
            }
            removed.push(relative.to_string());
        }
        Ok(())
    };
    for relative in TEMPLATE_ONLY {
        gone(relative)?;
    }
    if !flags.keep_history {
        for relative in HISTORY {
            gone(relative)?;
        }
        for (relative, text) in FRESH_INDEXES {
            let path = root.join(relative);
            if path.exists() {
                changed.extend(edit(root, &path, dry_run, |_| Ok((*text).to_string()))?);
            }
        }
    }

    changed.extend(edit(
        root,
        &root.join("src/identity.rs"),
        dry_run,
        |text| rewrite_identity(text, next),
    )?);
    changed.extend(edit(root, &root.join("Cargo.toml"), dry_run, |text| {
        Ok(rewrite_manifest(text, next))
    })?);

    let mut texts = Vec::new();
    walk(root, &|name| name.ends_with(".md"), &mut texts);
    walk(
        &root.join("tests"),
        &|name| name.ends_with(".rs"),
        &mut texts,
    );
    texts.extend(NAMED.iter().map(|relative| root.join(relative)));
    for path in texts.into_iter().filter(|path| path.exists()) {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        if removed
            .iter()
            .any(|gone| relative.starts_with(gone.as_str()))
        {
            continue;
        }
        changed.extend(edit(root, &path, dry_run, |text| {
            Ok(strip_template_only(&replace_all(text, &pairs)))
        })?);
    }

    changed.sort();
    changed.dedup();
    let mut report = vec![
        format!(
            "{} {ORG_NAME}/{REPO_NAME} -> {}/{}",
            if dry_run { "Would change" } else { "Changed" },
            next.org,
            next.repo
        ),
        format!(
            "  tool prefix  {} -> {}",
            *TOOL_PREFIX,
            pairs[0].1.trim_end_matches('_')
        ),
        format!("  server id    {SERVER_ID} -> {}", next.server_id),
    ];
    report.extend(changed.iter().map(|path| format!("  edited   {path}")));
    report.extend(removed.iter().map(|path| format!("  removed  {path}")));
    Ok(report)
}

/// Build the renamed server and list its tools: a prefix that pushes an ID past 64 characters
/// fails here, by name.
fn verify(root: &Path) -> Result<String, String> {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["run", "--quiet", "--bin"])
        .arg(identity_bin(root)?)
        .arg("--")
        .arg("tools")
        .current_dir(root)
        .output()
        .map_err(|error| format!("cannot run cargo: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "The renamed server does not build or list its tools:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// The binary's name, as the renamed `Cargo.toml` now declares it.
fn identity_bin(root: &Path) -> Result<String, String> {
    let manifest =
        fs::read_to_string(root.join("Cargo.toml")).map_err(|error| error.to_string())?;
    let bin = manifest
        .split("[[bin]]")
        .nth(1)
        .ok_or("Cargo.toml has no [[bin]]")?;
    let line = bin
        .lines()
        .find(|line| line.starts_with("name = "))
        .ok_or("[[bin]] has no name")?;
    Ok(line
        .trim_start_matches("name = ")
        .trim_matches('"')
        .to_string())
}

fn init(argv: &[String]) -> Result<(), String> {
    let flags = parse_args(argv)?;
    if flags.help {
        println!("{HELP}");
        return Ok(());
    }
    let root = root();
    if !flags.dry_run && !flags.force && is_template_itself(&root) {
        return Err(
            "origin is the template repository itself. Run init in the new repository made from it, or pass --force.".into(),
        );
    }
    let next = identity_for(&flags)?;
    println!("{}", rename(&root, &flags, &next)?.join("\n"));
    if flags.dry_run {
        return Ok(());
    }
    let tools = verify(&root)?;
    println!("\nTools:\n{tools}\nNext: edit README.md and content/, then run cargo test.");
    Ok(())
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let Some((task, rest)) = argv.split_first() else {
        eprintln!("Usage: cargo xtask init -- --org <org> --repo <repo>");
        return ExitCode::from(2);
    };
    if task != "init" {
        eprintln!("Unknown task \"{task}\". The only task is init.");
        return ExitCode::from(2);
    }
    match init(rest) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}
