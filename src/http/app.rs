//! The HTTP transport, as an axum [`Router`].
//!
//! This module builds the router and returns it. **It does not listen**: `src/http/mod.rs` owns
//! the port, so the router can be reasoned about, and tested, without binding one.
//!
//! The surface is deliberately narrow: `POST /mcp` and `GET /healthz`, a JSON-RPC 404 for
//! everything else, and a 405 for any other method on `/mcp`. **Everything except `GET /healthz`
//! needs the bearer token.** Nothing here is a second source of truth about the tool list: each
//! request gets a fresh [`SkillServer`] over the same static surface stdio serves.

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post_service};
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::auth::{BearerCheck, require_bearer, token_problem};
use super::host::{LOCALHOST, require_host};
use super::rpc_error::rpc_error;
use crate::identity::SERVER_ID;
use crate::server::SkillServer;
use crate::version::VERSION;

/// The request body ceiling, in bytes.
pub const BODY_LIMIT_BYTES: usize = 4 * 1024 * 1024;

/// What [`router`] takes.
#[derive(Debug, Clone, Default)]
pub struct AppOptions {
    /// The bearer token every request but `GET /healthz` must carry. Empty is no token, which
    /// only `local` allows.
    pub token: String,
    /// The `Host` allow-list. Empty applies no check, except on a tokenless local server.
    pub allowed_hosts: Vec<String>,
    /// Set when the server binds loopback. Only then may `token` be empty. It defaults to
    /// `false`, so leaving it out can never turn the token check off.
    pub local: bool,
    /// Cancelled on shutdown, so in-flight MCP requests end with the server.
    pub shutdown: CancellationToken,
}

/// The health check. Answers without a session, a request or a tool.
async fn healthz() -> Response {
    (
        StatusCode::OK,
        axum::Json(json!({ "status": "ok", "server": SERVER_ID, "version": VERSION })),
    )
        .into_response()
}

/// Any other method on `/mcp`. A refusal, not a 404: the path exists.
async fn method_not_allowed(request: Request) -> Response {
    rpc_error(
        StatusCode::METHOD_NOT_ALLOWED,
        -32000,
        format!("{} is not supported in stateless mode", request.method()),
    )
}

/// Anything that is not a route this server has.
async fn not_found(request: Request) -> Response {
    rpc_error(
        StatusCode::NOT_FOUND,
        -32601,
        format!("Not found: {}", request.uri()),
    )
}

/// Read the body, up to [`BODY_LIMIT_BYTES`], and refuse it unless it is JSON.
///
/// An oversized body and a malformed one get **the same 400 / `-32700`**, as they always have.
/// The body is read only after the `Host` and token checks, so a caller without the token
/// cannot make this server read up to 4 MiB.
async fn require_json(request: Request, next: Next) -> Response {
    let (parts, body) = request.into_parts();
    let bytes = match to_bytes(body, BODY_LIMIT_BYTES).await {
        Ok(bytes) if serde_json::from_slice::<serde_json::Value>(&bytes).is_ok() => bytes,
        _ => {
            return rpc_error(
                StatusCode::BAD_REQUEST,
                -32700,
                "Parse error: request body is not valid JSON",
            );
        }
    };
    next.run(Request::from_parts(parts, Body::from(bytes)))
        .await
}

/// The Skill MCP endpoint: `rmcp`'s Streamable HTTP service, stateless.
///
/// No sessions (`legacy_session_mode = false`, `NeverSessionManager`) and plain JSON responses,
/// so there is no session store to keep bounded. `rmcp`'s own `Host` check is off: the one in
/// `src/http/host.rs` runs ahead of every route instead.
fn mcp_service(
    server: SkillServer,
    shutdown: CancellationToken,
) -> StreamableHttpService<SkillServer, NeverSessionManager> {
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .with_max_request_body_bytes(BODY_LIMIT_BYTES)
        .with_cancellation_token(shutdown)
        .disable_allowed_hosts();
    StreamableHttpService::new(
        move || Ok(server),
        Arc::new(NeverSessionManager::default()),
        config,
    )
}

/// Build the HTTP router.
///
/// **There is no way to build it without a token unless `local` is set**: a missing or
/// too-short token is an error, so an unauthenticated HTTP server cannot come from forgetting an
/// option. `src/http/mod.rs` checks first, to report it before binding.
///
/// Layers run outermost first: the `Host` allow-list, then the token, then (on `/mcp`) the body
/// check, then the route.
pub fn router(server: SkillServer, options: AppOptions) -> Result<Router, String> {
    if let Some(problem) = token_problem(&options.token, !options.local) {
        return Err(format!("Refusing to build the HTTP app: {problem}."));
    }
    let open = options.token.is_empty();

    let mcp = post_service(mcp_service(server, options.shutdown.clone()))
        .layer(middleware::from_fn(require_json))
        .fallback(method_not_allowed);

    let mut app = Router::new()
        .route("/healthz", get(healthz))
        .route("/mcp", mcp)
        .fallback(not_found);

    if !open {
        let check = BearerCheck::new(&options.token);
        app = app.layer(middleware::from_fn(move |request, next| {
            require_bearer(check.clone(), request, next)
        }));
    }

    let hosts: Vec<String> = if !options.allowed_hosts.is_empty() {
        options.allowed_hosts.clone()
    } else if open {
        LOCALHOST.iter().map(|host| (*host).to_string()).collect()
    } else {
        Vec::new()
    };
    if !hosts.is_empty() {
        let allowed: Arc<[String]> = hosts.into();
        app = app.layer(middleware::from_fn(move |request, next| {
            require_host(allowed.clone(), request, next)
        }));
    }

    Ok(app)
}
