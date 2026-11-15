//! The package version. `Cargo.toml` is the only place it is written; Cargo hands it to the
//! compiler, so the binary cannot report anything else.

/// The version in `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
