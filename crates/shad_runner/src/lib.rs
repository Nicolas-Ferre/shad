//! Parser for the Shad programming language.
//!
//! # Examples
//!
//! ```rust
//! # use shad_runner::*;
//! #
//! # fn no_run() {
//! let mut runner = Runner::new("path/to/myscript.shd").unwrap();
//! runner.run_step();
//! # }
//! ```

mod runner;

pub use runner::*;
