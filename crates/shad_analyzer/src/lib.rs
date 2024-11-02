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
//!     let asg = Analysis::run(&ast);
//!     if asg.errors.is_empty() {
//!         println!("{asg:#?}")
//!
//!     } else {
//!         for err in &asg.errors {
//!             println!("{err}");
//!         }
//!     }
//! }
//! ```

mod analysis;
mod checks;
mod errors;
mod listing;
mod registration;
mod transformation;

pub use analysis::*;
pub use registration::buffers::*;
pub use registration::functions::*;
pub use registration::idents::*;
pub use registration::run_blocks::*;
pub use registration::shaders::*;
pub use registration::types::*;
