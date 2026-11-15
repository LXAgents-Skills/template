//! Who this server is. The one file a new repository edits to become itself.
//!
//! `cargo xtask init -- --org <org> --repo <repo>` rewrites the five constants below. Everything
//! else that names the server — the tool prefix, every tool ID, the health check, the startup
//! lines — is derived from them, so no other source file spells the org or the repo.

use std::sync::LazyLock;

/// The GitHub organization, as written there.
pub const ORG_NAME: &str = "LXAgents-Skills";

/// The repository name, as written there.
pub const REPO_NAME: &str = "template";

/// The server's name in `initialize`, `/healthz` and every log line.
pub const SERVER_ID: &str = "lxagents-template";

/// A human-readable title for clients that show one.
pub const SERVER_TITLE: &str = "LXAgents Template";

/// One sentence on what the set is; the server's `instructions` open with it.
pub const SERVER_SUMMARY: &str =
    "An example Skill MCP instruction set, served read-only. Replace content/ with your own set.";

/// A name as it appears inside a tool ID: lowercase, and every run of anything that is not an
/// ASCII letter or digit becomes one underscore, trimmed at both ends.
///
/// `LXAgents-Skills` is `lxagents_skills`; `shared-instruction` is `shared_instruction`.
pub fn id_part(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut gap = false;
    for ch in name.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            if gap && !out.is_empty() {
                out.push('_');
            }
            gap = false;
            out.push(ch);
        } else {
            gap = true;
        }
    }
    out
}

/// The prefix every tool ID carries: `{org}_{repo}`.
///
/// A client that connects several servers sees one flat tool list, so a bare `automation` from
/// two servers would collide. The prefix names the server a tool comes from. It is never
/// shortened: when an ID is too long, the base name is.
pub static TOOL_PREFIX: LazyLock<String> =
    LazyLock::new(|| format!("{}_{}", id_part(ORG_NAME), id_part(REPO_NAME)));

/// A base name with the prefix in front of it.
pub fn tool_id(base: &str) -> String {
    format!("{}_{base}", *TOOL_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_part_lowercases_and_collapses_separators() {
        assert_eq!(id_part("LXAgents-Skills"), "lxagents_skills");
        assert_eq!(id_part("shared-instruction"), "shared_instruction");
        assert_eq!(id_part("--a..b__c--"), "a_b_c");
        assert_eq!(id_part("my.repo_2"), "my_repo_2");
    }

    #[test]
    fn the_prefix_is_org_then_repo() {
        assert_eq!(
            *TOOL_PREFIX,
            format!("{}_{}", id_part(ORG_NAME), id_part(REPO_NAME))
        );
        assert_eq!(
            tool_id("automation"),
            format!("{}_automation", *TOOL_PREFIX)
        );
    }
}
