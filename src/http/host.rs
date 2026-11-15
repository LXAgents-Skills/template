//! The `Host` header allow-list.
//!
//! It runs ahead of the token check and every route, `/healthz` included: an allow-list that
//! guards `/mcp` and not `/healthz` has a hole in it. `rmcp`'s own `Host` check is turned off in
//! `src/http/app.rs` so this is the only one, and it applies to every route the same way.
//!
//! A tokenless loopback server gets the localhost list when none is configured: with no token,
//! the `Host` check is what stops a web page that rebinds its own name to 127.0.0.1 from reading
//! this server through the browser.

use std::sync::Arc;

use axum::extract::Request;
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;

use super::rpc_error::rpc_error;

/// The hosts a tokenless loopback server answers to when `MCP_ALLOWED_HOSTS` is unset.
pub const LOCALHOST: &[&str] = &["localhost", "127.0.0.1", "[::1]"];

/// The hostname in a `Host` header, without its port. `[::1]:3000` is `[::1]`.
pub fn hostname(host: &str) -> &str {
    if host.starts_with('[') {
        return host.find(']').map_or(host, |end| &host[..=end]);
    }
    host.split(':').next().unwrap_or(host)
}

/// Middleware that answers `403` unless the request's `Host` is in `allowed`.
pub async fn require_host(allowed: Arc<[String]>, request: Request, next: Next) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok());
    match host {
        Some(host)
            if allowed
                .iter()
                .any(|entry| entry.eq_ignore_ascii_case(hostname(host))) =>
        {
            next.run(request).await
        }
        Some(host) => rpc_error(
            StatusCode::FORBIDDEN,
            -32000,
            format!("Invalid Host: {}", hostname(host)),
        ),
        None => rpc_error(StatusCode::FORBIDDEN, -32000, "Missing Host header"),
    }
}

#[cfg(test)]
mod tests {
    use super::hostname;

    #[test]
    fn strips_the_port() {
        assert_eq!(hostname("localhost:3000"), "localhost");
        assert_eq!(hostname("example.com"), "example.com");
        assert_eq!(hostname("[::1]:3000"), "[::1]");
        assert_eq!(hostname("[::1]"), "[::1]");
    }
}
