extern crate jumpjet;

use std::process::Command;

#[cfg(test)]
fn main() {
    use std::env;

    let current_dir = env::current_dir().unwrap();

    let runtime_tests_dir = current_dir.join("tests/runtime-tests/");

    // The Jumpjet CLI is a sibling workspace member. `cargo test` only builds it
    // as a hashed test binary under `deps/`, not as the plain `jumpjet-cli`
    // executable, so build it explicitly before invoking it below.
    let cli_path = current_dir.join("../../target/debug/jumpjet-cli");
    let output = Command::new(env!("CARGO"))
        .args(["build", "-p", "jumpjet-cli"])
        .current_dir(&current_dir)
        .output()
        .expect("Failed to execute cargo build for jumpjet-cli");

    if !output.status.success() {
        panic!(
            "jumpjet-cli build failed!\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

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

    let output = Command::new(&cli_path)
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
