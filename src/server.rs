//! The Skill MCP server: an `rmcp` [`ServerHandler`] over the checked surface.
//!
//! The surface is [`crate::instructions::surface`] and nothing else. A tool listed anywhere else
//! is invisible to [`list_tools`], to the CLI and to the tests. No tool takes an argument, a verb
//! or a credential, and nothing here writes or reaches a network: the code that would is absent.

use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    JsonObject, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::{RequestContext, RoleServer};

use crate::identity::{SERVER_ID, SERVER_SUMMARY, SERVER_TITLE, tool_id};
use crate::instructions::Instruction;
use crate::version::VERSION;

/// The `instructions` a client receives in `initialize`.
pub fn instructions() -> String {
    [
        SERVER_SUMMARY.to_string(),
        format!(
            "Read {} once at the start of every session: it lists every tool and the one condition that activates each.",
            tool_id("automation")
        ),
        "Call no other tool until its condition is true. No tool takes an argument.".to_string(),
    ]
    .join(" ")
}

/// A registered tool, as the CLI prints it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSummary {
    pub name: String,
    pub description: String,
}

/// The registered tools, as name and description. The CLI prints this, so the two surfaces
/// cannot drift apart.
pub fn list_tools(surface: &[Instruction]) -> Vec<ToolSummary> {
    surface
        .iter()
        .map(|tool| ToolSummary {
            name: tool.name.clone(),
            description: tool.description.clone(),
        })
        .collect()
}

/// The input schema every tool shares: an object with no properties, because no tool takes an
/// argument.
fn no_arguments() -> Arc<JsonObject> {
    let mut schema = JsonObject::new();
    schema.insert("type".into(), "object".into());
    schema.insert("properties".into(), JsonObject::new().into());
    Arc::new(schema)
}

/// The server. Cheap to clone: it holds a reference to the static surface, so the HTTP transport
/// builds one per request without copying a file.
#[derive(Debug, Clone, Copy)]
pub struct SkillServer {
    surface: &'static [Instruction],
}

impl SkillServer {
    /// A server over `surface`.
    pub fn new(surface: &'static [Instruction]) -> Self {
        Self { surface }
    }

    /// Every tool, as the protocol lists it.
    pub fn tools(&self) -> Vec<Tool> {
        let schema = no_arguments();
        self.surface
            .iter()
            .map(|tool| Tool::new(tool.name.clone(), tool.description.clone(), schema.clone()))
            .collect()
    }

    /// The text of the tool called `name`, or `None` when there is no such tool.
    pub fn text_of(&self, name: &str) -> Option<&'static str> {
        self.surface
            .iter()
            .find(|tool| tool.name == name)
            .map(|tool| tool.text)
    }
}

impl ServerHandler for SkillServer {
    fn get_info(&self) -> ServerConfig {
        let implementation = Implementation::new(SERVER_ID, VERSION).with_title(SERVER_TITLE);
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(implementation)
            .with_instructions(instructions())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(self.tools()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        match self.text_of(&request.name) {
            Some(text) => Ok(CallToolResult::success(vec![ContentBlock::text(text)]).into()),
            None => Err(McpError::invalid_params(
                format!("Tool {} not found", request.name),
                None,
            )),
        }
    }
}
