//! How a file in `content/` gets its tool ID.
//!
//! ```text
//! {org}_{repo}_{base}
//! ```
//!
//! `{org}_{repo}` is `TOOL_PREFIX` from `src/identity.rs`. `{base}` is the filename: folder
//! dropped, `.md` dropped, lowercased, `-` turned into `_`. So `example/example-guide.md` is
//! `{org}_{repo}_example_guide`.
//!
//! Full rules, and when to add an override: `.agents/rules/tool-ids.md`.

use crate::identity::tool_id;

/// The longest ID this server publishes. The protocol allows 128, but several clients cap a
/// tool name at 64, so every ID is held to 64 at boot.
pub const MAX_ID_LENGTH: usize = 64;

/// Base names that do not survive mechanical derivation, keyed by set-relative path.
///
/// Use one when the derived ID would be longer than `MAX_ID_LENGTH`, when two folders hold the
/// same filename, or when the filename says nothing about the file. **Only the base is ever
/// shortened; the prefix is added by `tool_id` and never edited.** A key that matches no file
/// is a boot error, so a rename cannot leave an override matching nothing.
///
/// Example: `("references/javascript-typescript-react-web-frontend-security.md", "js_ts_react_web_frontend_security")`.
pub const OVERRIDE_BASES: &[(&str, &str)] = &[];

/// The override for `path` in `overrides`, if there is one.
fn override_for<'a>(path: &str, overrides: &[(&str, &'a str)]) -> Option<&'a str> {
    overrides
        .iter()
        .find(|(key, _)| *key == path)
        .map(|(_, base)| *base)
}

/// The base name a path derives on its own.
pub fn derived_base(relative_path: &str) -> String {
    let file = relative_path.rsplit('/').next().unwrap_or(relative_path);
    file.strip_suffix(".md")
        .unwrap_or(file)
        .to_lowercase()
        .replace('-', "_")
}

/// The tool ID for a set-relative path.
pub fn tool_name_for(relative_path: &str, overrides: &[(&str, &str)]) -> String {
    match override_for(relative_path, overrides) {
        Some(base) => tool_id(base),
        None => tool_id(&derived_base(relative_path)),
    }
}

/// Why `name` cannot be published, or `None` when it can.
pub fn name_problem(name: &str) -> Option<String> {
    let mut chars = name.chars();
    let valid = chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !valid {
        return Some(format!(
            "\"{name}\" is not a valid tool name (lowercase letters, digits and underscores, starting with a letter)"
        ));
    }
    if name.len() > MAX_ID_LENGTH {
        return Some(format!(
            "\"{name}\" is {} characters, over the {MAX_ID_LENGTH}-character cap",
            name.len()
        ));
    }
    None
}
