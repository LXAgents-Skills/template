//! The text edits `init` makes. Pure: each takes a file's text and returns the new text.

use crate::identity::Identity;

/// Set `pub const NAME: &str = …;` in `src/identity.rs`. The value may span lines; it ends at the
/// first `;` outside a string literal, and the whole value is replaced with a new literal.
fn set_constant(source: &str, name: &str, value: &str) -> Result<String, String> {
    let head = format!("pub const {name}: &str =");
    let start = source
        .find(&head)
        .ok_or_else(|| format!("src/identity.rs has no \"pub const {name}: &str\"."))?;
    let after = start + head.len();
    let end = literal_end(&source[after..])
        .map(|offset| after + offset)
        .ok_or_else(|| format!("{name} has no closing ';'."))?;
    Ok(format!(
        "{}{head} {value:?};{}",
        &source[..start],
        &source[end + 1..]
    ))
}

/// The offset of the `;` that ends a constant whose value starts at `text`: the first `;` outside
/// a string literal.
fn literal_end(text: &str) -> Option<usize> {
    let mut in_string = false;
    let mut escaped = false;
    for (at, ch) in text.char_indices() {
        match ch {
            _ if escaped => escaped = false,
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            ';' if !in_string => return Some(at),
            _ => {}
        }
    }
    None
}

/// `src/identity.rs` with the five identity constants replaced.
pub fn rewrite_identity(source: &str, identity: &Identity) -> Result<String, String> {
    let constants = [
        ("ORG_NAME", &identity.org),
        ("REPO_NAME", &identity.repo),
        ("SERVER_ID", &identity.server_id),
        ("SERVER_TITLE", &identity.title),
        ("SERVER_SUMMARY", &identity.summary),
    ];
    constants
        .iter()
        .try_fold(source.to_string(), |text, (name, value)| {
            set_constant(&text, name, value)
        })
}

/// `Cargo.toml` for the new repository: its description, its keywords without `template`, and
/// version `0.0.0`. The package and binary names and the links are the server ID and the slug,
/// which [`replace_all`] swaps everywhere.
pub fn rewrite_manifest(source: &str, identity: &Identity) -> String {
    source
        .lines()
        .map(|line| {
            if line.starts_with("description = ") {
                format!("description = {:?}", identity.summary)
            } else if line.starts_with("version = ") {
                "version = \"0.0.0\"".to_string()
            } else if line.starts_with("keywords = ") {
                line.replace(", \"template\"", "")
                    .replace("\"template\", ", "")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// Replace every occurrence of each `(from, to)` pair, in order.
pub fn replace_all(text: &str, pairs: &[(String, String)]) -> String {
    pairs.iter().fold(text.to_string(), |out, (from, to)| {
        if from == to {
            out
        } else {
            out.replace(from.as_str(), to)
        }
    })
}

/// Drop every `<!-- template-only -->` … `<!-- /template-only -->` block, markers included, with
/// the line break after it. Text that is true only of the template itself lives there.
pub fn strip_template_only(text: &str) -> String {
    const OPEN: &str = "<!-- template-only -->";
    const CLOSE: &str = "<!-- /template-only -->";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(OPEN) {
        let Some(close) = rest[start..].find(CLOSE) else {
            break;
        };
        out.push_str(&rest[..start]);
        let mut end = start + close + CLOSE.len();
        if rest[end..].starts_with("\r\n") {
            end += 2;
        } else if rest[end..].starts_with('\n') {
            end += 1;
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> Identity {
        Identity {
            org: "Acme".into(),
            repo: "skills".into(),
            server_id: "acme-skills".into(),
            title: "Acme \"Skills\"".into(),
            summary: "The skills; served.".into(),
        }
    }

    #[test]
    fn rewrites_every_constant_even_across_lines() {
        let source = "pub const ORG_NAME: &str = \"A\";\npub const REPO_NAME: &str = \"b\";\npub const SERVER_ID: &str = \"c\";\npub const SERVER_TITLE: &str = \"d\";\npub const SERVER_SUMMARY: &str =\n    \"e; f\";\nfn x() {}\n";
        let out = rewrite_identity(source, &identity()).unwrap();
        assert!(out.contains("pub const ORG_NAME: &str = \"Acme\";"));
        assert!(out.contains("pub const SERVER_TITLE: &str = \"Acme \\\"Skills\\\"\";"));
        assert!(
            out.contains("pub const SERVER_SUMMARY: &str = \"The skills; served.\";\nfn x() {}")
        );
        let again = rewrite_identity(&out, &identity()).unwrap();
        assert_eq!(again, out);
    }

    #[test]
    fn a_missing_constant_is_an_error() {
        assert!(
            rewrite_identity("", &identity())
                .unwrap_err()
                .contains("ORG_NAME")
        );
    }

    #[test]
    fn manifest() {
        let source = "[package]\nname = \"x\"\nversion = \"1.2.3\"\ndescription = \"old\"\nkeywords = [\"mcp\", \"template\"]\n";
        let out = rewrite_manifest(source, &identity());
        assert!(
            out.contains("version = \"0.0.0\"")
                && out.contains("description = \"The skills; served.\"")
        );
        assert!(out.contains("keywords = [\"mcp\"]"));
    }

    #[test]
    fn strips_template_only_blocks() {
        let text = "a\n<!-- template-only -->\nsecret\n<!-- /template-only -->\nb\n";
        assert_eq!(strip_template_only(text), "a\nb\n");
        assert_eq!(strip_template_only("no markers\n"), "no markers\n");
    }

    #[test]
    fn replaces_in_order() {
        let pairs = vec![
            ("a_".to_string(), "b_".to_string()),
            ("x".to_string(), "x".to_string()),
        ];
        assert_eq!(replace_all("a_1 a_2 x", &pairs), "b_1 b_2 x");
    }
}
