use std::path::PathBuf;
use std::fs;
use serde_json::Value as JsonValue;
use jsonschema::Validator; // Changed from JSONSchema to Validator in newer versions
use miette::Result; // Result from miette

use crate::error::AppError; // Assuming error.rs is in src/ and AppError is pub

// Function to load and compile a JSON schema from a PathBuf (local or URL)
pub fn load_and_compile_schema(schema_path: &PathBuf) -> Result<Validator, AppError> { // Changed return type
    let schema_content: String;
    let source_display = schema_path.to_string_lossy().to_string();

    if schema_path.starts_with("http://") || schema_path.starts_with("https://") {
        // Ensure to_str().unwrap() is safe or handle Option
        let url_str = schema_path.to_str().ok_or_else(|| AppError::InvalidSchemaPath {
            path_display: source_display.clone(),
        })?;
        schema_content = reqwest::blocking::get(url_str)
            .map_err(|e| AppError::SchemaFetchError { url: source_display.clone(), source: e })?
            .text()
            .map_err(|e| AppError::SchemaFetchError { url: source_display.clone(), source: e })?;
    } else {
        schema_content = fs::read_to_string(schema_path)
            .map_err(|e| AppError::FileReadError { path: schema_path.clone(), source: e, span: None })?;
    }

    let schema_json: JsonValue = serde_json::from_str(&schema_content)
        .map_err(|e| AppError::SchemaParseError { source_display: source_display.clone(), source: e })?;
    
    // Use Validator::new instead of JSONSchema::compile
    Validator::new(&schema_json)
        .map_err(|e| {
            AppError::SchemaCompileError {
                source_display,
                source: e, 
            }
        })
}
 