//! A new repository's identity, worked out from the flags. Pure.

use crate::args::Flags;

/// What `init` writes into `src/identity.rs` and `Cargo.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub org: String,
    pub repo: String,
    pub server_id: String,
    pub title: String,
    pub summary: String,
}

/// `my-repo` is `My Repo`.
fn title_case(name: &str) -> String {
    name.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether `name` is a GitHub name: letters, digits, `.`, `_` and `-`, starting with a letter or digit.
fn github_name(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// The full identity. `server_id`, `title` and `summary` default from the org and repo:
/// `LXAgents-Skills` and `security` give `lxagents-security`, the way the existing servers are named.
pub fn identity_for(flags: &Flags) -> Result<Identity, String> {
    let (Some(org), Some(repo)) = (flags.org.clone(), flags.repo.clone()) else {
        return Err("Both --org and --repo are required.".into());
    };
    if !github_name(&org) || !github_name(&repo) {
        return Err(
            "--org and --repo take GitHub names: letters, digits, '.', '_' and '-'.".into(),
        );
    }
    let brand = org
        .split(|c: char| !c.is_ascii_alphanumeric())
        .next()
        .unwrap_or(&org)
        .to_string();
    Ok(Identity {
        server_id: flags
            .server_id
            .clone()
            .unwrap_or_else(|| format!("{brand}-{repo}").to_lowercase()),
        title: flags
            .title
            .clone()
            .unwrap_or_else(|| format!("{brand} {}", title_case(&repo))),
        summary: flags
            .summary
            .clone()
            .unwrap_or_else(|| format!("The {repo} instruction set, served read-only.")),
        org,
        repo,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_from_org_and_repo() {
        let flags = Flags {
            org: Some("LXAgents-Skills".into()),
            repo: Some("my-skills".into()),
            ..Flags::default()
        };
        let identity = identity_for(&flags).unwrap();
        assert_eq!(identity.server_id, "lxagents-my-skills");
        assert_eq!(identity.title, "LXAgents My Skills");
        assert_eq!(
            identity.summary,
            "The my-skills instruction set, served read-only."
        );
    }

    #[test]
    fn refusals() {
        assert!(identity_for(&Flags::default()).is_err());
        let bad = Flags {
            org: Some("a b".into()),
            repo: Some("r".into()),
            ..Flags::default()
        };
        assert!(identity_for(&bad).is_err());
    }
}
