#![allow(missing_docs)]

fn main() {
    println!("cargo::rerun-if-changed=cases_valid");
    println!("cargo::rerun-if-changed=cases_invalid");
}
