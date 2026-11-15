//! The set as `build.rs` embedded it: one `(set-relative path, text)` pair per `.md` file.

include!(concat!(env!("OUT_DIR"), "/content.rs"));
