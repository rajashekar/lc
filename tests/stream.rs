use std::process::Command;

#[test]
fn test_lc_stream() {
    // Test that the --stream flag is recognized by the CLI
    // This is a simple synchronous test that checks if the flag is parsed correctly
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to run lc --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that the --stream flag is mentioned in help output
    assert!(
        stdout.contains("--stream") || stderr.contains("--stream"),
        "Expected --stream flag to be mentioned in help output. stdout: '{}', stderr: '{}'",
        stdout,
        stderr
    );

    // Also verify the command completed successfully
    assert!(output.status.success(), "Help command should succeed");
}
