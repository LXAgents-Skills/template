//! Serve over stdio.
//!
//! stdout is the JSON-RPC channel here, so nothing on this path writes to it, and nothing on it
//! reads `MCP_AUTH_TOKEN`: whoever can use the pipe already runs the process.

use rmcp::ServiceExt;

use crate::identity::SERVER_ID;
use crate::server::SkillServer;
use crate::version::VERSION;

/// Serve `server` on stdin and stdout until the client closes the pipe.
pub async fn serve(server: SkillServer) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let running = server.serve(rmcp::transport::stdio()).await?;
    eprintln!("{SERVER_ID} {VERSION} serving over stdio");
    running.waiting().await?;
    Ok(())
}
