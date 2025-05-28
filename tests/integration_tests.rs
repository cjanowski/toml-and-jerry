use std::process::Command;

#[test]
fn test_valid_json_file() {
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "test-examples/valid-config.json", "--schema", "test-examples/schema.json"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed for valid JSON file");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("All processed files are valid!"));
}

#[test]
fn test_invalid_json_file() {
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "test-examples/invalid-config.json", "--schema", "test-examples/schema.json"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail for invalid JSON file");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Schema validation failed"));
}

#[test]
fn test_valid_toml_file() {
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "test-examples/valid-config.toml", "--schema", "test-examples/schema.json"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed for valid TOML file");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("All processed files are valid!"));
}

#[test]
fn test_valid_yaml_file() {
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "test-examples/valid-config.yaml", "--schema", "test-examples/schema.json"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed for valid YAML file");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("All processed files are valid!"));
}

#[test]
fn test_json_output_format() {
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "test-examples/invalid-config.json", "--schema", "test-examples/schema.json", "--format", "json"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail for invalid file");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Find the JSON array in the output (skip the processing messages)
    let json_lines: Vec<&str> = stdout.lines()
        .skip_while(|line| !line.trim().starts_with("["))
        .collect();
    let json_output_str = json_lines.join("\n");
    
    // Parse the JSON output to ensure it's valid
    let json_output: serde_json::Value = serde_json::from_str(&json_output_str)
        .expect("Output should be valid JSON");
    
    assert!(json_output.is_array(), "JSON output should be an array");
    assert!(!json_output.as_array().unwrap().is_empty(), "JSON output should contain errors");
}

#[test]
fn test_sarif_output_format() {
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "test-examples/valid-config.json", "--schema", "test-examples/schema.json", "--format", "sarif"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed for valid file");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Find the JSON object in the output (skip the processing messages)
    let json_lines: Vec<&str> = stdout.lines()
        .skip_while(|line| !line.trim().starts_with("{"))
        .collect();
    let json_output_str = json_lines.join("\n");
    
    // Parse the SARIF output to ensure it's valid
    let sarif_output: serde_json::Value = serde_json::from_str(&json_output_str)
        .expect("SARIF output should be valid JSON");
    
    assert_eq!(sarif_output["version"], "2.1.0", "SARIF version should be 2.1.0");
    assert!(sarif_output["runs"].is_array(), "SARIF should have runs array");
}

#[test]
fn test_multiple_files() {
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "test-examples/valid-config.json", "test-examples/valid-config.toml", "test-examples/valid-config.yaml", "--schema", "test-examples/schema.json"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed for multiple valid files");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("All processed files are valid!"));
}

#[test]
fn test_nonexistent_file() {
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "test-examples/nonexistent.json", "--schema", "test-examples/schema.json"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail for nonexistent file");
}

#[test]
fn test_nonexistent_schema() {
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "test-examples/valid-config.json", "--schema", "test-examples/nonexistent-schema.json"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail for nonexistent schema");
} 