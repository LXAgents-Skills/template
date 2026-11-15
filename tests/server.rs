//! The surface as served: one tool per file on disk, under the prefix, returning the file whole.

mod common;

use std::fs;

use skill_server::identity::{SERVER_SUMMARY, TOOL_PREFIX, tool_id};
use skill_server::instructions::embedded::FILES;
use skill_server::instructions::naming::{OVERRIDE_BASES, tool_name_for};
use skill_server::instructions::surface;
use skill_server::server::{SkillServer, instructions, list_tools};

#[test]
fn every_file_on_disk_is_embedded_and_served() {
    let embedded: Vec<&str> = FILES.iter().map(|(path, _)| *path).collect();
    assert_eq!(embedded, common::content_files());
    let served: Vec<&str> = surface().unwrap().iter().map(|tool| tool.path).collect();
    assert_eq!(served, embedded);
}

#[test]
fn every_id_carries_the_prefix_and_matches_the_derivation() {
    for tool in surface().unwrap() {
        assert!(
            tool.name.starts_with(&format!("{}_", *TOOL_PREFIX)),
            "{}",
            tool.name
        );
        assert_eq!(tool.name, tool_name_for(tool.path, OVERRIDE_BASES));
    }
}

#[test]
fn each_tool_returns_its_file_byte_for_byte() {
    let server = SkillServer::new(surface().unwrap());
    for tool in surface().unwrap() {
        let on_disk = fs::read_to_string(common::root().join("content").join(tool.path)).unwrap();
        assert_eq!(server.text_of(&tool.name), Some(on_disk.as_str()));
    }
    assert_eq!(server.text_of("nope"), None);
}

#[test]
fn no_tool_takes_an_argument() {
    for tool in SkillServer::new(surface().unwrap()).tools() {
        assert_eq!(
            tool.input_schema.get("type").and_then(|v| v.as_str()),
            Some("object")
        );
        let properties = tool
            .input_schema
            .get("properties")
            .and_then(|v| v.as_object())
            .unwrap();
        assert!(properties.is_empty(), "{} takes arguments", tool.name);
    }
}

#[test]
fn the_hub_is_served_and_lists_every_other_tool() {
    let hub_name = tool_id("automation");
    let tools = list_tools(surface().unwrap());
    let hub = SkillServer::new(surface().unwrap())
        .text_of(&hub_name)
        .expect("content/automation.md");
    for tool in tools.iter().filter(|tool| tool.name != hub_name) {
        assert!(
            hub.contains(&tool.name),
            "content/automation.md has no row for {}",
            tool.name
        );
    }
}

#[test]
fn the_instructions_open_with_the_summary_and_name_the_hub() {
    let text = instructions();
    assert!(text.starts_with(SERVER_SUMMARY));
    assert!(text.contains(&tool_id("automation")));
}
