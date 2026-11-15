//! The ID rule: derivation, overrides, and every refusal the surface makes at boot.

use skill_server::identity::{TOOL_PREFIX, tool_id};
use skill_server::instructions::SurfaceError;
use skill_server::instructions::naming::{
    MAX_ID_LENGTH, derived_base, name_problem, tool_name_for,
};
use skill_server::instructions::surface::build;

const DESCRIBED: &str = "---\nname: x\ndescription: a file\n---\nbody\n";

#[test]
fn the_base_drops_the_folder_and_the_extension() {
    assert_eq!(
        derived_base("git/branching-strategy.md"),
        "branching_strategy"
    );
    assert_eq!(derived_base("Top-Level.md"), "top_level");
    assert_eq!(
        tool_name_for("example/example-guide.md", &[]),
        format!("{}_example_guide", *TOOL_PREFIX)
    );
}

#[test]
fn an_override_replaces_the_base_only() {
    let overrides = [("skills/github/api.md", "github_api")];
    assert_eq!(
        tool_name_for("skills/github/api.md", &overrides),
        tool_id("github_api")
    );
}

#[test]
fn name_problems() {
    assert!(name_problem("lxagents_ok_1").is_none());
    assert!(name_problem("1bad").is_some());
    assert!(name_problem("Bad").is_some());
    assert!(name_problem("has-dash").is_some());
    assert!(name_problem(&"a".repeat(MAX_ID_LENGTH)).is_none());
    assert!(
        name_problem(&"a".repeat(MAX_ID_LENGTH + 1))
            .unwrap()
            .contains("64-character cap")
    );
}

#[test]
fn refuses_an_id_over_the_cap() {
    let long = format!("{}.md", "x".repeat(MAX_ID_LENGTH));
    let files: Vec<(&'static str, &'static str)> =
        vec![(Box::leak(long.into_boxed_str()), DESCRIBED)];
    let error = build(&files, &[]).unwrap_err();
    assert!(matches!(error, SurfaceError::BadName { .. }), "{error}");
    assert!(error.to_string().contains("OVERRIDE_BASES"));
}

#[test]
fn refuses_a_collision_and_names_both_files() {
    let files = [("github/api.md", DESCRIBED), ("gitlab/api.md", DESCRIBED)];
    let error = build(&files, &[]).unwrap_err().to_string();
    assert!(
        error.contains("github/api.md") && error.contains("gitlab/api.md"),
        "{error}"
    );
    let fixed = build(&files, &[("github/api.md", "github_api")]).unwrap();
    assert_eq!(fixed.len(), 2);
}

#[test]
fn refuses_a_stale_override() {
    let error = build(&[("a.md", DESCRIBED)], &[("gone.md", "gone")]).unwrap_err();
    assert_eq!(error, SurfaceError::StaleOverride("gone.md".into()));
}

#[test]
fn refuses_a_missing_description() {
    let error = build(&[("a.md", "---\nname: a\n---\n")], &[]).unwrap_err();
    assert_eq!(error, SurfaceError::NoDescription("a.md".into()));
    let error = build(&[("b.md", "# no frontmatter\n")], &[]).unwrap_err();
    assert_eq!(error, SurfaceError::NoDescription("b.md".into()));
}

#[test]
fn refuses_an_empty_set() {
    assert_eq!(build(&[], &[]).unwrap_err(), SurfaceError::Empty);
}
