#[test]
fn test_cli_help() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "help"])
        .output()
        .expect("Failed to run command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("prep"));
}