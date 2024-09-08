//! Analyzer for the Shader programming language.
//!
//! This crate provides utilities to analyze parsed Shad code.
//!
//! # Examples
//!
//! ```rust
//! # use shad_parser::ParsedProgram;
//! #
//! # fn no_run() {
//! let parsed = shad_parser::ParsedProgram::parse_file("myapp.shd");
//! let analyzed = ParsedProgram::analyze(&parsed);
//! match analyzed {
//!     Ok(parsed) => println!("{parsed:#?}"),
//!     Err(err) => println!("{err}"),
//! }
//! # }
//! ```

mod buffer;
mod error;
mod init_compute_shaders;
mod program;
mod type_;

pub use buffer::*;
pub use error::*;
pub use init_compute_shaders::*;
pub use program::*;
pub use type_::*;
