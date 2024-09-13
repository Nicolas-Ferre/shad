//! Parser for the Shad programming language.
//!
//! # Examples
//!
//! ```rust
//! # use shad_runner::*;
//! #
//! # fn no_run() {
//! let runner = Runner::new("path/to/myscript.shd").unwrap();
//! runner.run();
//! # }
//! ```

mod runner;

pub use runner::*;
