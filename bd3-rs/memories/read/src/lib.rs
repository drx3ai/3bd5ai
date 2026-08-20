//! Read-path helpers for Bd3 memories.
//!
//! This crate owns memory injection, memory citation parsing, and telemetry
//! classification for read access to the memory folder. It intentionally does
//! not depend on the memory write pipeline.

pub mod citations;
mod metrics;
pub mod usage;

use bd3_utils_absolute_path::AbsolutePathBuf;

pub fn memory_root(bd3_home: &AbsolutePathBuf) -> AbsolutePathBuf {
    bd3_home.join("memories")
}
