//! Shad CLI.

use clap::Parser;
use shad::Args;

// coverage: off (not easy to test)

fn main() {
    Args::parse().run();
}
