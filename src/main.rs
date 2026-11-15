//! Entry point: parse the command line, then carry it out.
//!
//! This is the only file that starts anything: every module below it is inert, so each can be
//! tested without binding a port.

use std::collections::BTreeMap;
use std::io::Write;
use std::process::ExitCode;

use skill_server::cli::{Command, help, parse};
use skill_server::config::Config;
use skill_server::instructions::{Instruction, surface};
use skill_server::server::{SkillServer, list_tools};
use skill_server::version::VERSION;
use skill_server::{http, stdio};

/// The checked surface, or the reason it failed, already printed.
fn checked_surface() -> Result<&'static [Instruction], ExitCode> {
    surface().map_err(|error| {
        eprintln!("{error}");
        ExitCode::from(1)
    })
}

/// Every tool ID, its length and its description, one per line, aligned.
fn print_tools() -> ExitCode {
    let surface = match checked_surface() {
        Ok(surface) => surface,
        Err(code) => return code,
    };
    let tools = list_tools(surface);
    let width = tools.iter().map(|tool| tool.name.len()).max().unwrap_or(0);
    let mut out = std::io::stdout().lock();
    for tool in tools {
        let _ = writeln!(
            out,
            "{:width$}  {:>2}  {}",
            tool.name,
            tool.name.len(),
            tool.description
        );
    }
    ExitCode::SUCCESS
}

/// The tokio runtime the configuration asks for. One worker is a current-thread runtime.
fn runtime(workers: usize) -> std::io::Result<tokio::runtime::Runtime> {
    if workers <= 1 {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
    } else {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(workers)
            .enable_all()
            .build()
    }
}

/// Serve the transport the environment, with `overrides` on top, names.
fn serve(overrides: &BTreeMap<&'static str, String>) -> ExitCode {
    let config = match Config::from_lookup(|name| {
        overrides
            .get(name)
            .cloned()
            .or_else(|| std::env::var(name).ok())
    }) {
        Ok(config) => config,
        Err(problem) => {
            eprintln!("{problem}");
            return ExitCode::from(2);
        }
    };
    let server = match checked_surface() {
        Ok(surface) => SkillServer::new(surface),
        Err(code) => return code,
    };

    let workers = if config.is_http() { config.workers } else { 1 };
    let result = runtime(workers).map_err(Into::into).and_then(|runtime| {
        runtime.block_on(async {
            if config.is_http() {
                http::serve(server, &config).await
            } else {
                stdio::serve(server).await.map(|()| true)
            }
        })
    });
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse(&args) {
        Command::Version => {
            println!("{VERSION}");
            ExitCode::SUCCESS
        }
        Command::Help { exit } => {
            println!("{}", help());
            ExitCode::from(exit)
        }
        Command::Tools => print_tools(),
        Command::Serve(overrides) => serve(&overrides),
        Command::Unknown(command) => {
            eprintln!("Unknown command \"{command}\". Try --help.");
            ExitCode::from(2)
        }
    }
}
