//! A Skill MCP server: every markdown file in `content/` is served read-only as its own tool,
//! over stdio or Streamable HTTP.
//!
//! Every module is inert: nothing here starts, binds or forks until `src/main.rs` asks.

pub mod cli;
pub mod config;
pub mod http;
pub mod identity;
pub mod instructions;
pub mod server;
pub mod stdio;
pub mod version;
