//! `--name value` flags into [`Flags`]. Pure.

/// Parsed flags.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Flags {
    pub help: bool,
    pub dry_run: bool,
    pub keep_history: bool,
    pub force: bool,
    pub org: Option<String>,
    pub repo: Option<String>,
    pub server_id: Option<String>,
    pub title: Option<String>,
    pub summary: Option<String>,
}

/// Parse `argv`. A flag with no value, an unknown flag, or an argument that is not a flag is an
/// error rather than a guess. A lone `--`, as `cargo xtask init -- …` passes it, is skipped.
pub fn parse_args(argv: &[String]) -> Result<Flags, String> {
    let mut flags = Flags::default();
    let mut args = argv.iter().filter(|arg| arg.as_str() != "--");
    while let Some(arg) = args.next() {
        let switch = match arg.as_str() {
            "--help" | "-h" => &mut flags.help,
            "--dry-run" => &mut flags.dry_run,
            "--keep-history" => &mut flags.keep_history,
            "--force" => &mut flags.force,
            _ => {
                let slot = match arg.as_str() {
                    "--org" => &mut flags.org,
                    "--repo" => &mut flags.repo,
                    "--server-id" => &mut flags.server_id,
                    "--title" => &mut flags.title,
                    "--summary" => &mut flags.summary,
                    other if other.starts_with("--") => {
                        return Err(format!("Unknown flag \"{other}\"."));
                    }
                    other => return Err(format!("Unexpected argument \"{other}\".")),
                };
                let value = args.next().filter(|value| !value.starts_with("--"));
                *slot = Some(
                    value
                        .ok_or_else(|| format!("{arg} needs a value."))?
                        .clone(),
                );
                continue;
            }
        };
        *switch = true;
    }
    Ok(flags)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| (*arg).to_string()).collect()
    }

    #[test]
    fn values_and_switches() {
        let flags = parse_args(&args(&[
            "--",
            "--org",
            "Acme",
            "--repo",
            "skills",
            "--dry-run",
        ]))
        .unwrap();
        assert_eq!(flags.org.as_deref(), Some("Acme"));
        assert_eq!(flags.repo.as_deref(), Some("skills"));
        assert!(flags.dry_run && !flags.force);
    }

    #[test]
    fn refusals() {
        assert!(
            parse_args(&args(&["--org"]))
                .unwrap_err()
                .contains("needs a value")
        );
        assert!(
            parse_args(&args(&["--org", "--repo"]))
                .unwrap_err()
                .contains("needs a value")
        );
        assert!(
            parse_args(&args(&["stray"]))
                .unwrap_err()
                .contains("Unexpected")
        );
        assert!(
            parse_args(&args(&["--nope"]))
                .unwrap_err()
                .contains("Unknown")
        );
    }
}
