//! Bearer-token authentication for the HTTP transport.
//!
//! **The transport decides whether a token is needed.** stdio is a pipe that a client spawns on
//! its own machine, so whoever can use it already runs the process and there is nobody to
//! authenticate. HTTP is a socket that anyone who can reach the port can open, so every request
//! to it must prove the caller holds the token, and the server refuses to run HTTP without one
//! rather than running open and saying so in a log nobody reads.
//!
//! **Where the socket listens decides whether that is a network.** A server bound to loopback
//! (`127.0.0.1`, `localhost`, `::1`) can only be reached from this machine, the same trust a
//! stdio pipe has, so a token is optional there: a local install runs without one, and the
//! localhost `Host` allow-list in `src/http/host.rs` stops a web page reaching it by DNS
//! rebinding. Any other bind is a server, and it refuses to start without a token. A token that
//! is set is always enforced.
//!
//! Nothing in this file is reached on the stdio path.

use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use super::rpc_error::rpc_error;

/// The shortest token the server accepts, in characters.
///
/// The token is the only thing standing between a network and this server, so a value like
/// `test` is refused at startup instead of being discovered by a scanner. The documented command,
/// `openssl rand -hex 32`, produces 64.
pub const MIN_TOKEN_LENGTH: usize = 32;

/// The configured token, or an empty string when there is none.
///
/// Read at call time and never cached: a value captured once is whatever happened to be set
/// then. Surrounding whitespace is dropped, because a token copied out of a file often carries a
/// trailing newline and an HTTP header cannot.
pub fn configured_token() -> String {
    std::env::var("MCP_AUTH_TOKEN")
        .map(|token| token.trim().to_string())
        .unwrap_or_default()
}

/// Why `token` cannot be used, or `None` when it can.
///
/// The message names the variable and the fix and never the value, so it is safe to log.
/// `required: false` is the local case: no token at all is then fine, but a token that is set and
/// too short is still a problem, because it was meant to protect something and would not.
pub fn token_problem(token: &str, required: bool) -> Option<String> {
    if token.is_empty() {
        return required.then(|| "MCP_AUTH_TOKEN is not set".to_string());
    }
    let length = token.chars().count();
    (length < MIN_TOKEN_LENGTH).then(|| {
        format!("MCP_AUTH_TOKEN is {length} characters, and it must be at least {MIN_TOKEN_LENGTH}")
    })
}

/// The credential in an `Authorization: Bearer <credential>` header, or `None`.
///
/// The scheme is case-insensitive (RFC 7235). Anything else (another scheme, no credential, two
/// of them) is not a bearer credential.
pub fn bearer_credential(header: &str) -> Option<&str> {
    let (scheme, rest) = header.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let credential = rest.trim_start_matches(' ');
    let valid = !credential.is_empty() && !credential.chars().any(char::is_whitespace);
    valid.then_some(credential)
}

/// SHA-256 of a string, so two values of any length compare in constant time.
fn digest(value: &str) -> [u8; 32] {
    Sha256::digest(value.as_bytes()).into()
}

/// The token check, built once from a token that already passed [`token_problem`].
#[derive(Clone)]
pub struct BearerCheck {
    expected: [u8; 32],
}

impl BearerCheck {
    pub fn new(token: &str) -> Self {
        Self {
            expected: digest(token),
        }
    }

    /// Whether `credential` is the token. Constant-time over fixed-length digests, so neither the
    /// token's length nor how much of it a guess got right shows in how long the answer takes.
    pub fn accepts(&self, credential: &str) -> bool {
        digest(credential).ct_eq(&self.expected).into()
    }
}

/// Whether a request is the one route that needs no token: exactly `GET /healthz`.
///
/// An orchestrator's probe cannot send a token, and the route returns only
/// `{ status, server, version }`. `HEAD /healthz` and `/healthz/` are not it, and are refused
/// without a token, which is the safe direction to be wrong in.
fn exempt(request: &Request) -> bool {
    request.method() == axum::http::Method::GET && request.uri().path() == "/healthz"
}

/// Middleware that lets a request through only when it carries the token.
///
/// A refusal is a `401` in the JSON-RPC envelope with `WWW-Authenticate: Bearer`. A request that
/// sent a credential which did not match also gets `error="invalid_token"`, so a client can tell
/// a missing header from a wrong one. Neither answer reveals anything about the token.
pub async fn require_bearer(check: BearerCheck, request: Request, next: Next) -> Response {
    if exempt(&request) {
        return next.run(request).await;
    }
    let credential = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(bearer_credential);

    match credential {
        Some(credential) if check.accepts(credential) => next.run(request).await,
        Some(_) => refuse(
            "Bearer realm=\"mcp\", error=\"invalid_token\"",
            "Unauthorized: the token is not valid",
        ),
        None => refuse(
            "Bearer realm=\"mcp\"",
            "Unauthorized: send the token as Authorization: Bearer <token>",
        ),
    }
}

fn refuse(challenge: &'static str, message: &str) -> Response {
    let mut response = rpc_error(StatusCode::UNAUTHORIZED, -32001, message);
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static(challenge),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_rules() {
        assert_eq!(
            token_problem("", true).as_deref(),
            Some("MCP_AUTH_TOKEN is not set")
        );
        assert_eq!(token_problem("", false), None);
        assert!(
            token_problem("short", false)
                .unwrap()
                .contains("5 characters")
        );
        assert_eq!(token_problem(&"a".repeat(32), true), None);
    }

    #[test]
    fn bearer_parsing() {
        assert_eq!(bearer_credential("Bearer abc"), Some("abc"));
        assert_eq!(bearer_credential("bearer   abc"), Some("abc"));
        assert_eq!(bearer_credential("Basic abc"), None);
        assert_eq!(bearer_credential("Bearer"), None);
        assert_eq!(bearer_credential("Bearer "), None);
        assert_eq!(bearer_credential("Bearer a b"), None);
    }

    #[test]
    fn comparison() {
        let check = BearerCheck::new("secret");
        assert!(check.accepts("secret"));
        assert!(!check.accepts("secre"));
        assert!(!check.accepts("secret2"));
    }
}
