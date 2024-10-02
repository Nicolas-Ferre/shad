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
//!     let asg = Asg::analyze(&ast);
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

mod asg;
mod buffer;
mod errors;
mod expr;
mod function;
mod passes;
mod result;
mod shader;
mod statement;
mod type_;

pub use asg::*;
pub use buffer::*;
pub use expr::*;
pub use function::*;
pub use passes::buffer_listing::*;
pub use passes::function_listing::*;
pub use passes::type_resolving::*;
pub use result::*;
pub use shader::*;
pub use statement::*;
pub use type_::*;
