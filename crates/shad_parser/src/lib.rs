//! Parser for the Shader programming language.
//!
//! This crate provides utilities to parse Shad syntax.
//!
//! # Examples
//!
//! ```rust
//! # use shad_parser::Program;
//! #
//! # fn no_run() {
//! let parsed = Program::parse_file("myapp.shd");
//! match parsed {
//!     Ok(parsed) => println!("{parsed:#?}"),
//!     Err(err) => println!("{err}"),
//! }
//! # }
//! ```

mod atoms;
mod common;
mod expr;
mod items;
mod program;

pub use atoms::*;
pub use common::*;
pub use expr::*;
pub use items::*;
pub use program::*;
