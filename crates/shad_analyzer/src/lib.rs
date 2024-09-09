//! Analyzer for the Shad programming language.
//!
//! This crate provides utilities to analyze parsed Shad code.
//!
//! # Examples
//!
//! ```rust
//! # use shad_parser::*;
//! # use shad_analyzer::*;
//! #
//! fn analyze_shad_program(parsed: ParsedProgram) {
//!     let analyzed = AnalyzedProgram::analyze(&parsed);
//!     if analyzed.errors().next().is_some() {
//!         for err in analyzed.errors() {
//!             println!("{err}");
//!         }
//!     } else {
//!         println!("{parsed:#?}")
//!     }
//! }
//! ```

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
