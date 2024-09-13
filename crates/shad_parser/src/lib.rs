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
//!     let ast = Ast::from_file(path);
//!     match ast {
//!         Ok(ast) => println!("{ast:#?}"),
//!         Err(err) => println!("{err}"),
//!     }
//! }
//! ```

mod ast;
mod atom;
mod common;
mod expr;
mod item;

pub use ast::*;
pub use atom::*;
pub use expr::*;
pub use item::*;
