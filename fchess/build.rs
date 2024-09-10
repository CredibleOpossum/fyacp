use std::process::{Command, Stdio};

fn main() {
    println!("cargo:rerun-if-changed=../magic_generator/src/main.rs");

    let command = if cfg!(target_os = "linux") {
        ["sh", "-c"]
    } else {
        // Good luck anything else!
        ["cmd", "/C"]
    };
    let output = Command::new(command[0])
        .arg(command[1])
        .arg("cargo run --release --manifest-path=\"../magic_generator/Cargo.toml\"")
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute command");

    println!("cargo:warning={:?}", output);
}
