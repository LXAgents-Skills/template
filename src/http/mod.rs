//! Serve over Streamable HTTP: check the token, bind, report, and drain on a signal.

pub mod app;
pub mod auth;
pub mod host;
pub mod rpc_error;

use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::identity::SERVER_ID;
use crate::server::SkillServer;
use crate::version::VERSION;
use app::{AppOptions, router};
use auth::{MIN_TOKEN_LENGTH, configured_token, token_problem};

/// How long in-flight requests get after a signal before they are cut off.
const DRAIN_GRACE: Duration = Duration::from_secs(5);

/// Why the HTTP server did not run.
pub type HttpError = Box<dyn std::error::Error + Send + Sync>;

/// Resolves on the first SIGTERM or SIGINT, naming it.
async fn signal() -> &'static str {
    let interrupt = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("install the SIGTERM handler");
        tokio::select! {
            _ = interrupt => "SIGINT",
            _ = terminate.recv() => "SIGTERM",
        }
    }
    #[cfg(not(unix))]
    {
        let _ = interrupt.await;
        "SIGINT"
    }
}

/// The startup lines: where it listens and which controls are on. None ever carries the token.
fn report(config: &Config, token_set: bool) {
    let where_ = if config.host == "0.0.0.0" {
        "all interfaces"
    } else {
        config.host.as_str()
    };
    eprintln!(
        "{SERVER_ID} {VERSION} serving over http on :{}/mcp ({where_})",
        config.port
    );
    if token_set {
        eprintln!(
            "{SERVER_ID} {VERSION} bearer token required on every route except GET /healthz."
        );
        if config.allowed_hosts.is_empty() {
            // Said out loud, because the default is the unguarded one.
            eprintln!(
                "{SERVER_ID} {VERSION} MCP_ALLOWED_HOSTS is unset, so no Host header allow-list is applied."
            );
        }
    } else {
        let limit = if config.allowed_hosts.is_empty() {
            "localhost"
        } else {
            "MCP_ALLOWED_HOSTS"
        };
        eprintln!(
            "{SERVER_ID} {VERSION} bound to loopback, so no bearer token is required; Host is limited to {limit}."
        );
    }
}

/// Serve `server` over HTTP until a signal. Returns `Ok(false)` when it refused to start.
///
/// Fail closed, before binding: HTTP without a token on anything but loopback is an open door on
/// a network, and the only thing to do about it is not to start.
pub async fn serve(server: SkillServer, config: &Config) -> Result<bool, HttpError> {
    let token = configured_token();
    if let Some(problem) = token_problem(&token, !config.local) {
        eprintln!(
            "{SERVER_ID} {VERSION} will not start the HTTP transport: {problem}. Set MCP_AUTH_TOKEN to a secret of at \
             least {MIN_TOKEN_LENGTH} characters, for example the output of: openssl rand -hex 32. stdio, and HTTP \
             bound to 127.0.0.1, need no token."
        );
        return Ok(false);
    }

    let shutdown = CancellationToken::new();
    let options = AppOptions {
        token: token.clone(),
        allowed_hosts: config.allowed_hosts.clone(),
        local: config.local,
        shutdown: shutdown.clone(),
    };
    let app = router(server, options)?;

    let listener = tokio::net::TcpListener::bind((config.host.as_str(), config.port)).await?;
    report(config, !token.is_empty());

    // Stop accepting first, so a request accepted during the drain gets an answer; then give
    // whatever is still in flight a short grace, after which the process exits rather than
    // waiting on it forever.
    let drained = shutdown.clone();
    let graceful = async move {
        let name = signal().await;
        eprintln!("{SERVER_ID} {VERSION} {name}, draining");
        drained.cancel();
        tokio::spawn(async {
            tokio::time::sleep(DRAIN_GRACE).await;
            std::process::exit(0);
        });
    };
    axum::serve(listener, app)
        .with_graceful_shutdown(graceful)
        .await?;
    Ok(true)
}
