//! The HTTP router: the routes, the token, the `Host` allow-list and every refusal.

use std::process::Command;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use skill_server::http::app::{AppOptions, BODY_LIMIT_BYTES, router};
use skill_server::identity::{SERVER_ID, tool_id};
use skill_server::instructions::surface;
use skill_server::server::SkillServer;
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

fn app(token: &str, allowed_hosts: &[&str], local: bool) -> Router {
    let options = AppOptions {
        token: token.into(),
        allowed_hosts: allowed_hosts
            .iter()
            .map(|host| (*host).to_string())
            .collect(),
        local,
        ..AppOptions::default()
    };
    router(SkillServer::new(surface().unwrap()), options).unwrap()
}

fn request(method: &str, path: &str, auth: Option<&str>, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, "localhost:3000");
    if let Some(auth) = auth {
        builder = builder.header(header::AUTHORIZATION, auth);
    }
    match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "application/json, text/event-stream")
            .body(Body::from(body.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, axum::http::HeaderMap, Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, body)
}

fn bearer() -> String {
    format!("Bearer {TOKEN}")
}

fn initialize() -> Value {
    json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}})
}

#[tokio::test]
async fn healthz_needs_no_token() {
    let (status, headers, body) = send(
        &app(TOKEN, &[], false),
        request("GET", "/healthz", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({"status":"ok","server":SERVER_ID,"version":"0.0.0"})
    );
    assert!(headers.get("x-powered-by").is_none() && headers.get(header::SERVER).is_none());
}

#[tokio::test]
async fn only_exactly_get_healthz_is_exempt() {
    let app = app(TOKEN, &[], false);
    for (method, path) in [
        ("HEAD", "/healthz"),
        ("GET", "/healthz/"),
        ("POST", "/healthz"),
    ] {
        let (status, _, _) = send(&app, request(method, path, None, None)).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path}");
    }
}

#[tokio::test]
async fn a_missing_token_and_a_wrong_one_are_told_apart() {
    let app = app(TOKEN, &[], false);
    let (status, headers, body) =
        send(&app, request("POST", "/mcp", None, Some(initialize()))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer realm=\"mcp\"");
    assert_eq!(body["error"]["code"], -32001);

    let (status, headers, _) = send(
        &app,
        request("POST", "/mcp", Some("Bearer wrong"), Some(initialize())),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        headers[header::WWW_AUTHENTICATE],
        "Bearer realm=\"mcp\", error=\"invalid_token\""
    );
}

#[tokio::test]
async fn routes_are_hidden_from_a_caller_without_the_token() {
    let app = app(TOKEN, &[], false);
    let (status, _, _) = send(&app, request("GET", "/nowhere", None, None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = send(&app, request("GET", "/mcp", None, None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn with_the_token_mcp_answers() {
    let app = app(TOKEN, &[], false);
    let (status, _, body) = send(
        &app,
        request("POST", "/mcp", Some(&bearer()), Some(initialize())),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"]["serverInfo"]["name"], SERVER_ID);

    let list = json!({"jsonrpc":"2.0","id":2,"method":"tools/list"});
    let (status, _, body) = send(&app, request("POST", "/mcp", Some(&bearer()), Some(list))).await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = body["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&tool_id("automation").as_str()), "{names:?}");

    let call = json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":tool_id("automation"),"arguments":{}}});
    let (_, _, body) = send(&app, request("POST", "/mcp", Some(&bearer()), Some(call))).await;
    let text = body["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.starts_with("---\n"));
}

#[tokio::test]
async fn other_methods_on_mcp_are_405_and_unknown_paths_404() {
    let app = app(TOKEN, &[], false);
    for method in ["GET", "DELETE", "PUT"] {
        let (status, _, body) = send(&app, request(method, "/mcp", Some(&bearer()), None)).await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED, "{method}");
        assert_eq!(body["error"]["code"], -32000);
    }
    let (status, _, body) = send(&app, request("GET", "/nowhere", Some(&bearer()), None)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], -32601);
    assert_eq!(body["error"]["message"], "Not found: /nowhere");
}

#[tokio::test]
async fn malformed_and_oversized_bodies_are_parse_errors() {
    let app = app(TOKEN, &[], false);
    let bad = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header(header::HOST, "localhost")
        .header(header::AUTHORIZATION, bearer())
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{not json"))
        .unwrap();
    let (status, _, body) = send(&app, bad).await;
    assert_eq!(
        (status, body["error"]["code"].clone()),
        (StatusCode::BAD_REQUEST, json!(-32700))
    );

    let huge = format!("\"{}\"", "a".repeat(BODY_LIMIT_BYTES + 1));
    let big = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header(header::HOST, "localhost")
        .header(header::AUTHORIZATION, bearer())
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(huge))
        .unwrap();
    let (status, _, body) = send(&app, big).await;
    assert_eq!(
        (status, body["error"]["code"].clone()),
        (StatusCode::BAD_REQUEST, json!(-32700))
    );
}

#[tokio::test]
async fn a_tokenless_local_server_answers_only_to_localhost() {
    let app = app("", &[], true);
    let (status, _, _) = send(&app, request("POST", "/mcp", None, Some(initialize()))).await;
    assert_eq!(status, StatusCode::OK);

    let rebound = Request::builder()
        .uri("/healthz")
        .header(header::HOST, "evil.example:3000")
        .body(Body::empty())
        .unwrap();
    let (status, _, body) = send(&app, rebound).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["message"], "Invalid Host: evil.example");
}

#[tokio::test]
async fn the_allow_list_guards_healthz_too() {
    let app = app(TOKEN, &["mcp.example"], false);
    let (status, _, _) = send(&app, request("GET", "/healthz", None, None)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let allowed = Request::builder()
        .uri("/healthz")
        .header(header::HOST, "mcp.example")
        .body(Body::empty())
        .unwrap();
    let (status, _, _) = send(&app, allowed).await;
    assert_eq!(status, StatusCode::OK);
}

#[test]
fn the_router_refuses_to_build_without_a_token_off_loopback() {
    let server = SkillServer::new(surface().unwrap());
    assert!(router(server, AppOptions::default()).is_err());
    let short = AppOptions {
        token: "short".into(),
        local: true,
        ..AppOptions::default()
    };
    assert!(router(server, short).is_err());
}

fn serve_http(host: &str, token: Option<&str>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lxagents-template"));
    command
        .args(["serve", "--http", "--host", host, "--port", "0"])
        .env_remove("MCP_AUTH_TOKEN");
    if let Some(token) = token {
        command.env("MCP_AUTH_TOKEN", token);
    }
    command.output().unwrap()
}

#[test]
fn the_process_will_not_start_open_on_a_network() {
    let output = serve_http("0.0.0.0", None);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("MCP_AUTH_TOKEN is not set"), "{stderr}");
    assert!(output.stdout.is_empty());
}

#[test]
fn the_process_refuses_a_short_token_even_on_loopback() {
    let output = serve_http("127.0.0.1", Some("short-token"));
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("11 characters") && !stderr.contains("short-token"),
        "{stderr}"
    );
}
