//! The served set: what is in `content/`, how each file is named, and the checks it must pass.

pub mod embedded;
pub mod frontmatter;
pub mod naming;
pub mod surface;

use std::sync::LazyLock;

pub use surface::{Instruction, SurfaceError};

/// The surface built from the embedded set and `OVERRIDE_BASES`, checked once.
pub static SURFACE: LazyLock<Result<Vec<Instruction>, SurfaceError>> =
    LazyLock::new(|| surface::build(embedded::FILES, naming::OVERRIDE_BASES));

/// The checked surface, or the reason the set cannot be served.
pub fn surface() -> Result<&'static [Instruction], &'static SurfaceError> {
    SURFACE.as_deref()
}
