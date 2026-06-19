extern crate jumpjet;

use jumpjet::runtime;
use std::process::Command;

#[cfg(test)]
fn main() {
    use std::{env, path::Path};

    let current_dir = env::current_dir().unwrap();

    let runtime_tests_dir = current_dir.join("tests/runtime-tests/");

    let output = Command::new("cargo")
        .arg("build")
        .current_dir(&runtime_tests_dir)
        .output()
        .expect("Failed to execute cargo build");

    if !output.status.success() {
        panic!(
            "cargo build failed!\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // let output = Command::new("jumpjet")
    let output = Command::new(current_dir.join("../../target/debug/jumpjet-cli"))
        .arg("build")
        .current_dir(&runtime_tests_dir)
        .output()
        .expect("Failed to execute Jumpjet build");

    if !output.status.success() {
        panic!(
            "Jumpjet build failed!\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let input_path = runtime_tests_dir.join("bin");
    let binary = std::fs::read(input_path.join("entrypoint.wasm")).unwrap();
    pollster::block_on(jumpjet::runtime::test(input_path.to_path_buf(), binary));
}
