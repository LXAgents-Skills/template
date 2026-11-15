//! Pull the single-line scalar fields out of a markdown file's frontmatter.

use std::collections::BTreeMap;

/// The fields of the frontmatter block, or `None` when there is none.
///
/// Only `key: value` on one line is read. A folded or multi-line YAML value would need a real
/// parser, and a dependency added for one call site is not worth it; a `description:` this
/// misses is reported as missing at boot rather than served empty.
pub fn parse_frontmatter(text: &str) -> Option<BTreeMap<String, String>> {
    let rest = text
        .strip_prefix("---\r\n")
        .or_else(|| text.strip_prefix("---\n"))?;
    let mut fields = BTreeMap::new();
    for line in rest.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.starts_with("---") {
            return Some(fields);
        }
        if let Some((key, value)) = field(line) {
            fields.insert(key.to_string(), value.trim().to_string());
        }
    }
    None
}

/// `key: value`, where the key is an identifier. Anything else is not a field.
fn field(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once(':')?;
    let mut chars = key.chars();
    let first = chars.next()?;
    let valid = (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    valid.then_some((key, value))
}

#[cfg(test)]
mod tests {
    use super::parse_frontmatter;

    #[test]
    fn reads_single_line_fields() {
        let fields = parse_frontmatter("---\nname: a\ndescription:  b c \n---\nbody").unwrap();
        assert_eq!(fields["name"], "a");
        assert_eq!(fields["description"], "b c");
    }

    #[test]
    fn reads_crlf() {
        let fields = parse_frontmatter("---\r\ndescription: x\r\n---\r\n").unwrap();
        assert_eq!(fields["description"], "x");
    }

    #[test]
    fn no_block_is_none() {
        assert!(parse_frontmatter("# title\n").is_none());
        assert!(parse_frontmatter("---\nname: a\n").is_none());
    }

    #[test]
    fn a_folded_value_reads_as_empty() {
        let fields = parse_frontmatter("---\ndescription: >\n  folded\n---\n").unwrap();
        assert_eq!(fields["description"], ">");
        let fields = parse_frontmatter("---\ndescription:\n  folded\n---\n").unwrap();
        assert_eq!(fields["description"], "");
    }
}
