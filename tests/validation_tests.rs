use std::path::PathBuf;
use toml_and_jerry::validation::{validate_inputs, PrintableError};
use toml_and_jerry::error::AppError;
use jsonschema::Validator;
use serde_json::Value as JsonValue;

// Helper function to create a validator from schema file
fn create_validator_from_schema_file(schema_path: &str) -> Validator {
    let schema_content = std::fs::read_to_string(schema_path)
        .expect("Failed to read schema file");
    let schema: JsonValue = serde_json::from_str(&schema_content)
        .expect("Failed to parse schema JSON");
    Validator::new(&schema).expect("Failed to create validator")
}

#[cfg(test)]
mod validation_unit_tests {
    use super::*;

    #[test]
    fn test_validate_inputs_with_valid_files() {
        let validator = create_validator_from_schema_file("test-examples/schema.json");
        let input_files = vec![
            PathBuf::from("test-examples/valid-config.json"),
            PathBuf::from("test-examples/valid-config.toml"),
            PathBuf::from("test-examples/valid-config.yaml"),
        ];

        let result = validate_inputs(input_files, &validator);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(errors.is_empty(), "Should have no validation errors for valid files");
    }

    #[test] 
    fn test_validate_inputs_with_invalid_schema_validation() {
        let validator = create_validator_from_schema_file("test-examples/schema.json");
        let input_files = vec![
            PathBuf::from("test-examples/invalid-config.json"),
            PathBuf::from("test-examples/missing-required-fields.json"),
        ];

        let result = validate_inputs(input_files, &validator);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(!errors.is_empty(), "Should have validation errors for invalid files");
        
        // Check that we get schema validation errors
        assert!(errors.iter().any(|e| matches!(e, AppError::SchemaValidationError { .. })));
    }

    #[test]
    fn test_validate_inputs_with_parse_errors() {
        let validator = create_validator_from_schema_file("test-examples/schema.json");
        let input_files = vec![
            PathBuf::from("test-examples/invalid-syntax.json"),
            PathBuf::from("test-examples/invalid-syntax.toml"),
            PathBuf::from("test-examples/invalid-syntax.yaml"),
        ];

        let result = validate_inputs(input_files, &validator);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(!errors.is_empty(), "Should have parse errors for malformed files");
        
        // Check that we get different types of parse errors
        assert!(errors.iter().any(|e| matches!(e, AppError::JsonParseError { .. })));
        assert!(errors.iter().any(|e| matches!(e, AppError::TomlParseError { .. })));
        assert!(errors.iter().any(|e| matches!(e, AppError::YamlParseError { .. })));
    }

    #[test]
    fn test_validate_inputs_with_nonexistent_file() {
        let validator = create_validator_from_schema_file("test-examples/schema.json");
        let input_files = vec![
            PathBuf::from("test-examples/nonexistent-file.json"),
        ];

        let result = validate_inputs(input_files, &validator);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(!errors.is_empty(), "Should have file read error for nonexistent file");
        
        // Check that we get a file read error
        assert!(errors.iter().any(|e| matches!(e, AppError::FileReadError { .. })));
    }

    #[test]
    fn test_validate_inputs_with_empty_file() {
        let validator = create_validator_from_schema_file("test-examples/schema.json");
        let input_files = vec![
            PathBuf::from("test-examples/empty-file.json"),
        ];

        let result = validate_inputs(input_files, &validator);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(!errors.is_empty(), "Should have parse error for empty JSON file");
        
        // Check that we get a JSON parse error for empty file
        assert!(errors.iter().any(|e| matches!(e, AppError::JsonParseError { .. })));
    }

    #[test] 
    fn test_printable_error_conversion() {
        // Test conversion from AppError to PrintableError
        let app_error = AppError::SchemaValidationError {
            path: PathBuf::from("test.json"),
            message: "Schema validation failed".to_string(),
            source_code: "{}".to_string(),
            error_span: miette::SourceSpan::new(0.into(), 1usize.into()),
            label_message: "Invalid field".to_string(),
            instance_path: "/name".to_string(),
            kind: "Required".to_string(),
        };

        let printable_error = PrintableError::from(&app_error);
        
        assert_eq!(printable_error.file_path, "test.json");
        assert_eq!(printable_error.error_type, "Schema validation error in file \"test.json\"");
        assert!(printable_error.message.contains("Schema validation failed"));
        assert_eq!(printable_error.json_path, Some("/name".to_string()));
        assert_eq!(printable_error.rule_id, "app::schema::validation_error");
    }

    #[test]
    fn test_validate_inputs_with_unsupported_extension() {
        let validator = create_validator_from_schema_file("test-examples/schema.json");
        
        // Create a temporary file with unsupported extension
        std::fs::write("test-examples/temp-file.xml", r#"<?xml version="1.0"?><root/>"#)
            .expect("Failed to create temp file");
        
        let input_files = vec![
            PathBuf::from("test-examples/temp-file.xml"),
        ];

        let result = validate_inputs(input_files, &validator);
        assert!(result.is_ok());
        let errors = result.unwrap();
        // Should have no errors because unsupported files are skipped
        assert!(errors.is_empty(), "Unsupported files should be skipped without errors");
        
        // Clean up
        std::fs::remove_file("test-examples/temp-file.xml").ok();
    }

    #[test]
    fn test_validate_inputs_with_type_errors() {
        let validator = create_validator_from_schema_file("test-examples/schema.json");
        let input_files = vec![
            PathBuf::from("test-examples/invalid-types.toml"),
        ];

        let result = validate_inputs(input_files, &validator);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(!errors.is_empty(), "Should have validation errors for wrong types");
        
        // Should get schema validation errors for type mismatches
        assert!(errors.iter().any(|e| matches!(e, AppError::SchemaValidationError { .. })));
    }
}

#[cfg(test)]
mod hcl_validation_tests {
    use super::*;

    #[test] 
    fn test_validate_hcl_valid_file() {
        let validator = create_validator_from_schema_file("test-examples/schema.json");
        let input_files = vec![
            PathBuf::from("test-examples/valid-config.hcl"),
        ];

        let result = validate_inputs(input_files, &validator);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(errors.is_empty(), "Valid HCL file should pass validation");
    }

    #[test]
    fn test_validate_hcl_invalid_schema() {
        let validator = create_validator_from_schema_file("test-examples/schema.json");
        let input_files = vec![
            PathBuf::from("test-examples/invalid-config.hcl"),
        ];

        let result = validate_inputs(input_files, &validator);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(!errors.is_empty(), "Invalid HCL file should fail validation");
        
        // Should get schema validation errors
        assert!(errors.iter().any(|e| matches!(e, AppError::SchemaValidationError { .. })));
    }

    #[test]
    fn test_validate_hcl_parse_error() {
        let validator = create_validator_from_schema_file("test-examples/schema.json");
        let input_files = vec![
            PathBuf::from("test-examples/invalid-syntax.hcl"),
        ];

        let result = validate_inputs(input_files, &validator);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(!errors.is_empty(), "Malformed HCL file should have parse errors");
        
        // Should get HCL parse errors
        assert!(errors.iter().any(|e| matches!(e, AppError::HclParseError { .. })));
    }
} 