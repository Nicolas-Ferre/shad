//! Parser for the Shad programming language.
//!
//! This crate provides utilities to parse Shad syntax.
//!
//! # Examples
//!
//! ```rust
//! # use shad_parser::*;
//! #
//! fn parse_shad_program(path: &str) {
//!     let parsed = ParsedProgram::parse_file(path);
//!     match parsed {
//!         Ok(parsed) => println!("{parsed:#?}"),
//!         Err(err) => println!("{err}"),
//!     }
//! }
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
