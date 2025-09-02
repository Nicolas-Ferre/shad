//! Compiler and runner of Shad programming language.

mod cli;
mod compilation;
mod exec;
mod language;

pub use cli::*;
pub use compilation::error::*;
pub use compilation::reading::*;
pub use compilation::transpilation::*;
pub use compilation::*;
pub use exec::runner::*;
