//! Every environment variable the server reads, read in one place.
//!
//! The one exception is `MCP_AUTH_TOKEN`, which `src/http/auth.rs` reads at call time on
//! purpose: a secret captured once at load is whatever was set then, and the token checks must
//! see it change.

/// The server's configuration, as [`Config::from_lookup`] returns it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// `MCP_TRANSPORT`, lowercased; `stdio` when unset.
    pub transport: String,
    /// `PORT`; 3000 when unset.
    pub port: u16,
    /// `HOST`; `0.0.0.0` when unset or empty.
    pub host: String,
    /// Whether `host` is loopback. A local HTTP server needs no token; see `src/http/auth.rs`.
    pub local: bool,
    /// `MCP_CLUSTER_WORKERS`: tokio worker threads. See [`worker_count`].
    pub workers: usize,
    /// `MCP_ALLOWED_HOSTS`, split on commas. Empty applies no `Host` check.
    pub allowed_hosts: Vec<String>,
}

/// A comma-separated list, trimmed, with empty entries dropped. Unset is empty.
fn split_list(raw: Option<&str>) -> Vec<String> {
    raw.unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(String::from)
        .collect()
}

/// How many tokio worker threads the HTTP server runs.
///
/// `MCP_CLUSTER_WORKERS` sets the count, and keeps the name it had when it counted forked
/// processes, so an existing deployment means the same thing. **1 is a current-thread runtime**:
/// one thread, nothing spawned. Unset, unparsable or 0, the count is the number of CPUs the
/// process was actually given, so a container with two CPUs gets two threads.
pub fn worker_count(raw: Option<&str>) -> usize {
    match raw.and_then(|value| value.trim().parse::<usize>().ok()) {
        Some(count) if count > 0 => count,
        _ => std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1),
    }
}

/// Whether `host` is a loopback address, which only this machine can reach.
///
/// `localhost`, any `127.x.x.x`, and `::1` with or without brackets. `0.0.0.0` and `::` mean
/// every interface, so they are not loopback; neither is an empty value.
pub fn is_loopback(host: &str) -> bool {
    let value = host.trim().to_ascii_lowercase();
    if value == "localhost" || value == "::1" || value == "[::1]" {
        return true;
    }
    let parts: Vec<&str> = value.split('.').collect();
    parts.len() == 4
        && parts[0] == "127"
        && parts
            .iter()
            .all(|part| (1..=3).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_digit()))
}

impl Config {
    /// The configuration from the process environment.
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// The configuration from `lookup`, which returns a variable's value or `None`.
    ///
    /// The bind address is named rather than implied: `0.0.0.0` is what a published port needs
    /// and what a container gets, but it is a decision, so it is made here, visibly.
    /// `HOST=127.0.0.1` is the way to take it back.
    ///
    /// An unset or empty `MCP_ALLOWED_HOSTS` applies no `Host` check rather than one guessed at:
    /// a wrong allow-list silently refusing every request is a worse failure than an absent one,
    /// and the absence is reported on startup.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let host = lookup("HOST")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "0.0.0.0".into());
        let port = match lookup("PORT").filter(|value| !value.is_empty()) {
            None => 3000,
            Some(raw) => raw
                .trim()
                .parse::<u16>()
                .map_err(|_| format!("PORT is \"{raw}\", and it must be a port number"))?,
        };
        Ok(Self {
            transport: lookup("MCP_TRANSPORT")
                .unwrap_or_else(|| "stdio".into())
                .to_lowercase(),
            port,
            local: is_loopback(&host),
            host,
            workers: worker_count(lookup("MCP_CLUSTER_WORKERS").as_deref()),
            allowed_hosts: split_list(lookup("MCP_ALLOWED_HOSTS").as_deref()),
        })
    }

    /// Whether the configured transport is HTTP.
    pub fn is_http(&self) -> bool {
        self.transport == "http" || self.transport == "streamable-http"
    }
}
