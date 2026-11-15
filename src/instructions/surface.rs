//! The tool surface, derived from the set: one tool per `.md` file.
//!
//! Adding a markdown file to `content/` is adding a tool. There is no tool module to write and
//! no list to edit. Everything is checked here, once, so a malformed set stops the process
//! instead of answering the first caller wrongly.

use std::collections::BTreeMap;
use std::fmt;

use super::frontmatter::parse_frontmatter;
use super::naming::{name_problem, tool_name_for};

/// One served file: the tool's name and description, the file it serves, and its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub name: String,
    pub description: String,
    pub path: &'static str,
    pub text: &'static str,
}

/// Why a set cannot be served. Each message names the file and the fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceError {
    StaleOverride(String),
    BadName {
        path: String,
        problem: String,
    },
    Collision {
        path: String,
        other: String,
        name: String,
    },
    NoDescription(String),
    Empty,
}

impl fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleOverride(key) => write!(
                f,
                "OVERRIDE_BASES names {key}, which is not in the set. Remove the override or restore the file."
            ),
            Self::BadName { path, problem } => write!(
                f,
                "{path}: {problem}. Shorten its base name in OVERRIDE_BASES (src/instructions/naming.rs)."
            ),
            Self::Collision { path, other, name } => {
                write!(
                    f,
                    "{path} and {other} both derive \"{name}\". Give one an entry in OVERRIDE_BASES."
                )
            }
            Self::NoDescription(path) => write!(
                f,
                "{path} has no frontmatter `description:`. It would publish a tool no client can route on."
            ),
            Self::Empty => write!(
                f,
                "No markdown files under content/. This server would expose nothing."
            ),
        }
    }
}

impl std::error::Error for SurfaceError {}

/// Build the surface from `files`, as `(set-relative path, text)` pairs sorted by path.
///
/// Each tool returns its file whole, frontmatter included, byte for byte. No tool takes an
/// argument, so there is no path for a caller to traverse with.
pub fn build(
    files: &[(&'static str, &'static str)],
    overrides: &[(&str, &str)],
) -> Result<Vec<Instruction>, SurfaceError> {
    if let Some((key, _)) = overrides
        .iter()
        .find(|(key, _)| !files.iter().any(|(path, _)| path == key))
    {
        return Err(SurfaceError::StaleOverride((*key).to_string()));
    }

    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    let mut tools = Vec::with_capacity(files.len());

    for &(path, text) in files {
        let name = tool_name_for(path, overrides);
        if let Some(problem) = name_problem(&name) {
            return Err(SurfaceError::BadName {
                path: path.into(),
                problem,
            });
        }
        if let Some(other) = seen.get(&name) {
            return Err(SurfaceError::Collision {
                path: path.into(),
                other: (*other).into(),
                name,
            });
        }
        let description = parse_frontmatter(text)
            .and_then(|fields| fields.get("description").cloned())
            .filter(|description| !description.is_empty())
            .ok_or_else(|| SurfaceError::NoDescription(path.into()))?;

        seen.insert(name.clone(), path);
        tools.push(Instruction {
            name,
            description,
            path,
            text,
        });
    }

    if tools.is_empty() {
        return Err(SurfaceError::Empty);
    }
    Ok(tools)
}
