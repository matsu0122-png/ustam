//! Demonstrates common `ustam` invocations.
//!
//! Run with: cargo run --example basic_usage
//!
//! `ustam` is a binary crate with no library target, so this example
//! cannot call its internals directly. Instead it runs the compiled
//! binary as a subprocess via `cargo run --bin ustam`, which mirrors
//! how a user would actually invoke the tool from a shell.

use std::process::Command;

fn run(args: &[&str]) {
    println!("$ ustam {}", args.join(" "));

    let output = Command::new(env!("CARGO"))
        .args(["run", "--quiet", "--bin", "ustam", "--"])
        .args(args)
        .output()
        .expect("failed to run ustam");

    print!("{}", String::from_utf8_lossy(&output.stdout));
    println!();
}

fn main() {
    run(&["src"]);
    run(&["-a", "src"]);
    run(&["-l", "src"]);
    run(&["-s", "-l", "src"]);
    run(&["-al", "src"]);
    run(&["--help"]);
}
