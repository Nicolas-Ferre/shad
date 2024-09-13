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
mod expr;
mod shader;
mod type_;

pub use asg::*;
pub use buffer::*;
pub use expr::*;
pub use shader::*;
pub use type_::*;
