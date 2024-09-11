//! Analyzer for the Shad programming language.
//!
//! This crate provides utilities to analyze a Shad AST.
//!
//! # Examples
//!
//! ```rust
//! # use shad_parser::*;
//! # use shad_analyzer::*;
//! #
//! fn analyze_shad_program(ast: Ast) {
//!     let analyzed = AnalyzedProgram::analyze(&ast);
//!     if analyzed.errors().next().is_some() {
//!         for err in analyzed.errors() {
//!             println!("{err}");
//!         }
//!     } else {
//!         println!("{analyzed:#?}")
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
