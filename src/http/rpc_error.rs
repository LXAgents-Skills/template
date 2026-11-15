//! A JSON-RPC error response.
//!
//! The shape every refusal on this server uses: the same envelope a client parses for a
//! successful response, so a client never has to branch on content type to find out it was
//! refused. The 401, the 403, the 404, the 405 and the parse errors all come through here, so
//! they cannot drift apart.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// `{"jsonrpc":"2.0","error":{"code":…,"message":…},"id":null}` with `status`.
pub fn rpc_error(status: StatusCode, code: i64, message: impl Into<String>) -> Response {
    let body = json!({ "jsonrpc": "2.0", "error": { "code": code, "message": message.into() }, "id": null });
    (status, Json(body)).into_response()
}
