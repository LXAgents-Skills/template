//! The command line, parsed: what the arguments ask for. Pure: arguments in, a [`Command`] out.
//!
//! `src/main.rs` carries the command out. Only the CLI's own commands write to stdout: on stdio,
//! stdout is the JSON-RPC channel.

use std::collections::BTreeMap;

use crate::identity::SERVER_ID;

/// The help text.
pub fn help() -> String {
    format!(
        "{SERVER_ID} - a Skill MCP instruction set, served read-only

Usage
  {SERVER_ID} [command]

Commands
  (none)           Run the Skill MCP server, configured by the environment
  serve            Run the Skill MCP server
                   [--stdio | --http] [--port N] [--host ADDR]
  tools            List every tool ID, its length, and its description

Options
  -h, --help       Show this help
  -v, --version    Show the version

Environment
  MCP_TRANSPORT        stdio (default) or http
  MCP_AUTH_TOKEN       bearer token, 32+ characters; required for http unless HOST is
                       loopback (127.0.0.1, localhost, ::1); unused by stdio
  PORT                 HTTP port, default 3000
  HOST                 HTTP interface, default 0.0.0.0
  MCP_ALLOWED_HOSTS    comma-separated Host allow-list
  MCP_CLUSTER_WORKERS  HTTP worker threads, default one per CPU; 1 is a single thread"
    )
}

/// What the arguments ask for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Serve, with these environment variables overriding the process environment.
    Serve(BTreeMap<&'static str, String>),
    /// Print every tool.
    Tools,
    /// Print the help; `exit` is the exit code.
    Help { exit: u8 },
    /// Print the version.
    Version,
    /// An unknown command.
    Unknown(String),
}

/// The value after `flag` in `args`, if any.
fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|at| args.get(at + 1))
}

/// Environment overrides from `serve` flags.
fn serve_overrides(args: &[String]) -> BTreeMap<&'static str, String> {
    let mut overrides = BTreeMap::new();
    if args.iter().any(|arg| arg == "--http") {
        overrides.insert("MCP_TRANSPORT", "http".to_string());
    }
    if args.iter().any(|arg| arg == "--stdio") {
        overrides.insert("MCP_TRANSPORT", "stdio".to_string());
    }
    if let Some(port) = flag_value(args, "--port") {
        overrides.insert("PORT", port.clone());
    }
    if let Some(host) = flag_value(args, "--host") {
        overrides.insert("HOST", host.clone());
    }
    overrides
}

/// Parse the arguments after the program name.
pub fn parse(args: &[String]) -> Command {
    let Some((command, rest)) = args.split_first() else {
        return Command::Serve(BTreeMap::new());
    };
    match command.as_str() {
        "-v" | "--version" => Command::Version,
        "-h" | "--help" | "help" => Command::Help { exit: 0 },
        "serve" => Command::Serve(serve_overrides(rest)),
        "tools" => Command::Tools,
        other => Command::Unknown(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| (*arg).to_string()).collect()
    }

    #[test]
    fn no_command_serves_from_the_environment() {
        assert_eq!(parse(&[]), Command::Serve(BTreeMap::new()));
    }

    #[test]
    fn serve_flags_become_overrides() {
        let Command::Serve(overrides) = parse(&args(&[
            "serve",
            "--http",
            "--port",
            "4000",
            "--host",
            "127.0.0.1",
        ])) else {
            panic!("not serve");
        };
        assert_eq!(overrides["MCP_TRANSPORT"], "http");
        assert_eq!(overrides["PORT"], "4000");
        assert_eq!(overrides["HOST"], "127.0.0.1");
    }

    #[test]
    fn other_commands() {
        assert_eq!(parse(&args(&["--version"])), Command::Version);
        assert_eq!(parse(&args(&["help"])), Command::Help { exit: 0 });
        assert_eq!(parse(&args(&["tools"])), Command::Tools);
        assert_eq!(parse(&args(&["nope"])), Command::Unknown("nope".into()));
    }
}
