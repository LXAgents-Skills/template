//! The environment, read in one place.

use std::collections::HashMap;

use skill_server::config::{Config, is_loopback, worker_count};

fn config(pairs: &[(&str, &str)]) -> Result<Config, String> {
    let env: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| ((*k).into(), (*v).into()))
        .collect();
    Config::from_lookup(|name| env.get(name).cloned())
}

#[test]
fn defaults() {
    let config = config(&[]).unwrap();
    assert_eq!(config.transport, "stdio");
    assert_eq!(config.port, 3000);
    assert_eq!(config.host, "0.0.0.0");
    assert!(!config.local);
    assert!(config.allowed_hosts.is_empty());
    assert!(config.workers >= 1);
}

#[test]
fn reads_every_variable() {
    let config = config(&[
        ("MCP_TRANSPORT", "HTTP"),
        ("PORT", "4100"),
        ("HOST", "127.0.0.1"),
        ("MCP_CLUSTER_WORKERS", "3"),
        ("MCP_ALLOWED_HOSTS", " a.example , ,b.example,"),
    ])
    .unwrap();
    assert!(config.is_http());
    assert_eq!(config.port, 4100);
    assert!(config.local);
    assert_eq!(config.workers, 3);
    assert_eq!(config.allowed_hosts, vec!["a.example", "b.example"]);
}

#[test]
fn a_bad_port_is_an_error_naming_the_variable() {
    assert!(config(&[("PORT", "eighty")]).unwrap_err().contains("PORT"));
}

#[test]
fn loopback() {
    for host in [
        "localhost",
        "127.0.0.1",
        "127.1.2.3",
        "::1",
        "[::1]",
        " LOCALHOST ",
    ] {
        assert!(is_loopback(host), "{host}");
    }
    for host in [
        "0.0.0.0",
        "::",
        "",
        "10.0.0.1",
        "127.0.0",
        "127.0.0.1.5",
        "localhost.evil.com",
    ] {
        assert!(!is_loopback(host), "{host}");
    }
}

#[test]
fn workers() {
    assert_eq!(worker_count(Some("1")), 1);
    assert_eq!(worker_count(Some("4")), 4);
    assert!(worker_count(Some("0")) >= 1);
    assert!(worker_count(Some("x")) >= 1);
    assert!(worker_count(None) >= 1);
}
