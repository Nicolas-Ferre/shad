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
mod errors;
mod items;
mod passes;
mod result;

pub use asg::*;
pub use items::buffer::*;
pub use items::expr::*;
pub use items::function::*;
pub use items::shader::*;
pub use items::statement::*;
pub use items::type_::*;
pub use passes::buffer_listing::*;
pub use passes::fn_listing::*;
pub use passes::type_resolving::*;
pub use result::*;
