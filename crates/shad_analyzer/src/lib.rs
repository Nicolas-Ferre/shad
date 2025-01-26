//! Analyzer for the Shad programming language.
//!
//! This crate provides utilities to analyze a Shad AST.
//!
//! # Examples
//!
//! ```rust
//! # use fxhash::*;
//! # use shad_parser::*;
//! # use shad_analyzer::*;
//! #
//! fn analyze_shad_program(asts: FxHashMap<String, Ast>) {
//!     let asg = Analysis::run(asts);
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
mod resolving;
mod transformation;

pub use analysis::*;
pub use listing::functions::*;
pub use registration::buffers::*;
pub use registration::const_fns::*;
pub use registration::constants::*;
pub use registration::functions::*;
pub use registration::generics::*;
pub use registration::run_blocks::*;
pub use registration::shaders::*;
pub use registration::types::*;
pub use registration::vars::*;
