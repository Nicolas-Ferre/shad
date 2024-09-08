//! Analyzer for the Shad programming language.
//!
//! This crate provides utilities to analyze parsed Shad code.

mod buffer;
mod error;
mod init_compute_shaders;
mod program;
mod type_;

pub use buffer::*;
pub use error::*;
pub use init_compute_shaders::*;
pub use program::*;
pub use type_::*;
