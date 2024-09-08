//! Parser for the Shader programming language.
//!
//! This crate provides utilities to parse Shad syntax.
//!
//! # Examples
//!
//! ```rust
//! # use shad_parser::ParsedProgram;
//! #
//! # fn no_run() {
//! let parsed = ParsedProgram::parse_file("myapp.shd");
//! match parsed {
//!     Ok(parsed) => println!("{parsed:#?}"),
//!     Err(err) => println!("{err}"),
//! }
//! # }
//! ```

mod atom;
mod common;
mod error;
mod expr;
mod item;
mod program;

pub use atom::*;
pub use common::*;
pub use error::*;
pub use expr::*;
pub use item::*;
pub use program::*;
