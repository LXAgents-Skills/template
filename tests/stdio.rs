//! stdio, end to end: the built binary answers JSON-RPC and writes nothing else to stdout.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use skill_server::identity::{SERVER_ID, tool_id};

#[test]
fn initialize_list_and_call_over_stdio() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lxagents-template"))
        .env("MCP_TRANSPORT", "stdio")
        .env_remove("MCP_AUTH_TOKEN")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();
    let messages = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":tool_id("automation"),"arguments":{}}}),
    ];
    for message in &messages {
        writeln!(stdin, "{message}").unwrap();
    }

    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut replies: Vec<Value> = Vec::new();
    while replies.len() < 3 {
        let line = lines.next().expect("a reply").unwrap();
        replies.push(
            serde_json::from_str(&line)
                .unwrap_or_else(|_| panic!("stdout carried non-JSON: {line}")),
        );
    }
    drop(stdin);
    child.wait().unwrap();

    assert_eq!(replies[0]["result"]["serverInfo"]["name"], SERVER_ID);
    assert_eq!(replies[0]["result"]["serverInfo"]["version"], "0.0.0");
    assert!(replies[1]["result"]["tools"].as_array().unwrap().len() >= 2);
    let text = replies[2]["result"]["content"][0]["text"].as_str().unwrap();
    let on_disk = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/content/automation.md"
    ))
    .unwrap();
    assert_eq!(text, on_disk);
}

#[test]
fn version_and_tools_on_the_command_line() {
    let bin = env!("CARGO_BIN_EXE_lxagents-template");
    let version = Command::new(bin).arg("--version").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&version.stdout).trim(), "0.0.0");

    let tools = Command::new(bin).arg("tools").output().unwrap();
    assert!(tools.status.success());
    let listing = String::from_utf8_lossy(&tools.stdout);
    assert!(
        listing
            .lines()
            .any(|line| line.starts_with(&format!("{} ", tool_id("automation")))),
        "{listing}"
    );

    let unknown = Command::new(bin).arg("nope").output().unwrap();
    assert_eq!(unknown.status.code(), Some(2));
}
