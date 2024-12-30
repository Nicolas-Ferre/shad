//! Parser for the Shad programming language.
//!
//! This crate provides utilities to parse Shad syntax.
//!
//! # Examples
//!
//! ```rust
//! # use shad_parser::*;
//! #
//! fn parse_shad_program(folder_path: &str) {
//!     let ast = Ast::from_dir(folder_path);
//!     match ast {
//!         Ok(ast) => println!("{ast:#?}"),
//!         Err(err) => println!("{err}"),
//!     }
//! }
//! ```

mod ast;
mod atom;
mod expr;
mod fn_call;
mod item;
mod statement;
mod token;
mod visit;

pub use ast::*;
pub use atom::*;
pub use expr::*;
pub use fn_call::*;
pub use item::buffer::*;
pub use item::constant::*;
pub use item::function::*;
pub use item::generics::*;
pub use item::gpu::*;
pub use item::import::*;
pub use item::run_block::*;
pub use item::struct_::*;
pub use item::*;
pub use statement::*;
pub use visit::*;
