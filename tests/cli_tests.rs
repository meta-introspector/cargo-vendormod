use std::process::Command;

#[test]
fn test_main_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-vendormod"))
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Usage: cargo-vendormod [OPTIONS] [COMMAND]"));
}
