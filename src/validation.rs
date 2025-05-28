use std::path::PathBuf;
use std::fs;
use serde_json::Value as JsonValue;
use jsonschema::Validator;
use miette::{Result, SourceSpan, Diagnostic};
use json_spanned_value::spanned::Value as SpannedJsonValue;
use toml_edit::{DocumentMut, Item as TomlEditItem, Value as TomlEditValue};
use serde::Serialize;

use crate::error::AppError;

// Helper function to convert json_spanned_value span tuple to miette::SourceSpan
fn convert_json_span(span_tuple: (usize, usize)) -> SourceSpan {
    let (start, end) = span_tuple;
    let length = if end > start { end - start } else { 1 };
    SourceSpan::new(start.into(), length.into())
}

// Function to find span for a JSON pointer path - simplified version that returns the span tuple
fn find_span_for_json_path(current_value: &SpannedJsonValue, path: &str) -> Option<(usize, usize)> {
    if path.is_empty() || path == "/" { // Root element
        return Some(current_value.span());
    }
    
    // For now, return the span of the root value as a fallback
    // TODO: Implement proper path traversal for json-spanned-value
    Some(current_value.span())
}

// Helper to convert toml_edit::Span to miette::SourceSpan
fn convert_toml_edit_span(toml_span: Option<std::ops::Range<usize>>) -> Option<SourceSpan> {
    toml_span.map(|range| {
        let length = if range.end > range.start { range.end - range.start } else { 1 };
        SourceSpan::new(range.start.into(), length.into())
    })
}

// Function to find span for a JSON pointer path in a TOML document
fn find_span_for_toml_path(mut current_item: &TomlEditItem, path: &str) -> Option<std::ops::Range<usize>> {
    if path.is_empty() || path == "/" {
        return current_item.span();
    }
    let segments = path.strip_prefix('/')?.split('/');

    for segment in segments {
        match current_item {
            TomlEditItem::Table(table) => {
                current_item = table.get(segment)?;
            }
            TomlEditItem::ArrayOfTables(array) => {
                let _index = segment.parse::<usize>().ok()?;
                // For ArrayOfTables, getting a specific table and then its span is complex.
                // The span of the whole array might be the best we can do easily here or the first table.
                // Let's return the span of the array itself if path points into it.
                // Or, if we need a specific table, we'd get array.get(index)?.span().
                // For now, let's assume the path will point to a value within a table or a direct value.
                // This part might need refinement based on how jsonschema reports paths for array of tables.
                return array.span(); // Simplification: span of the whole array of tables
            }
            TomlEditItem::Value(value) => {
                match value {
                    TomlEditValue::Array(array) => {
                        let _index = segment.parse::<usize>().ok()?;
                        // For TomlEditValue::Array, each element is a TomlEditValue, not an Item directly.
                        // We need to get the specific TomlValue then its span if available.
                        // TomlEditValue itself doesn't have a direct .span() like Item.
                        // The array.get(index) gives a TomlEditValue. Its span comes from the Array's own formatting.
                        // This is tricky. The span of the whole array might be the most practical.
                        return array.span(); // Span of the whole array value
                    }
                    TomlEditValue::InlineTable(table) => {
                        // Inline tables are values. To get a sub-item, we would need to treat it like a table item.
                        // This requires a temporary TomlEditItem::Table if possible, or careful handling.
                        // For now, if path goes into an inline table, return span of the inline table itself.
                        return table.span(); // Span of the whole inline table
                    }
                    _ => return None, // Path goes deeper, but current value is not a container type with named/indexed children
                }
            }
            _ => return None, // Not a table or array of tables, cannot go deeper with named segments.
        }
    }
    current_item.span()
}

#[derive(Serialize)] // Ensure PrintableError can be serialized to JSON
#[serde(rename_all = "camelCase")]
pub struct PrintableError { // Made PrintableError public
    pub file_path: String,
    pub error_type: String, // e.g., "YamlParseError", "SchemaValidationError"
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub json_path: Option<String>, // For schema validation errors
    pub rule_id: String, // From AppError diagnostic code
}

impl From<&AppError> for PrintableError {
    fn from(app_error: &AppError) -> Self {
        let line = None;
        let column = None;
        let mut json_path = None;
        let error_type = app_error.to_string().split_once(':').map_or_else(|| "UnknownError".to_string(), |(et, _)| et.to_string());
        let rule_id = app_error.code().map_or_else(|| "N/A".to_string(), |c| c.to_string());
        let message = match app_error {
            AppError::YamlParseError { span: _, .. } |
            AppError::JsonParseError { span: _, .. } |
            AppError::TomlParseError { span: _, .. } |
            AppError::HclParseError { span: _, .. } => {
                // For miette SourceSpan, we don't directly get line/col easily without source code context.
                // This is a simplification. A more robust way would be to calculate line/col from offset and source.
                // For now, we are not populating line/column from these parse errors directly here.
                // The primary message from the error itself (e.g., e.to_string()) is used.
                app_error.to_string() // Or a more specific message field if available
            }
            AppError::SchemaValidationError { instance_path,  .. } => {
                json_path = Some(instance_path.clone());
                // The main message for SchemaValidationError is already formatted in its creation.
                app_error.to_string()
            }
            _ => app_error.to_string(),
        };
        
        // Try to extract line/column from spans if possible (simplistic for now)
        match app_error {
            AppError::YamlParseError { span: _, .. } |
            AppError::JsonParseError { span: _, .. } |
            AppError::TomlParseError { span: _, .. } |
            AppError::HclParseError { span: _, .. } |
            AppError::SchemaValidationError { error_span: _, .. } => {
                // This is a placeholder. True line/col from SourceSpan needs the source text.
                // We will use the diagnostic information from miette for this, if possible,
                // or pass the source text to this conversion.
                // For a simple JSON report now, we might omit line/col or make them optional.
            }
            _ => {}
        }

        PrintableError {
            file_path: match app_error {
                AppError::FileReadError { path, .. } => path.to_string_lossy().into_owned(),
                AppError::SchemaFetchError { url, .. } => url.clone(),
                AppError::SchemaParseError { source_display, .. } => source_display.clone(),
                AppError::SchemaCompileError { source_display, .. } => source_display.clone(),
                AppError::YamlParseError { path, .. } => path.to_string_lossy().into_owned(),
                AppError::SchemaValidationError { path, .. } => path.to_string_lossy().into_owned(),
                AppError::JsonParseError { path, .. } => path.to_string_lossy().into_owned(),
                AppError::TomlParseError { path, .. } => path.to_string_lossy().into_owned(),
                AppError::HclParseError { path, .. } => path.to_string_lossy().into_owned(),
                AppError::InvalidSchemaPath { path_display } => path_display.clone(),
            },
            error_type,
            message,
            line, // Will be None for now mostly
            column, // Will be None for now mostly
            json_path,
            rule_id,
        }
    }
}

pub fn validate_inputs(
    inputs: Vec<PathBuf>,
    compiled_schema: &Validator,
) -> Result<Vec<AppError>, AppError> { // format_arg removed, main will handle formatting
    
    let mut collected_errors: Vec<AppError> = Vec::new();

    for input_path in inputs {
        println!("Processing file: {:?}", input_path);

        let extension = input_path.extension().and_then(|ext| ext.to_str());
        let file_content = match fs::read_to_string(&input_path) {
            Ok(c) => c,
            Err(e) => {
                collected_errors.push(AppError::FileReadError {
                    path: input_path.clone(),
                    source: e,
                    span: None, 
                });
                continue; 
            }
        };

        match extension {
            Some("yaml") | Some("yml") => {
                match serde_yaml::from_str::<serde_yaml::Value>(&file_content) {
                    Ok(parsed_yaml) => {
                        let json_value_for_validation: JsonValue = match serde_yaml::from_value(parsed_yaml) {
                            Ok(v) => v,
                            Err(_convert_err) => {
                                let err_span = SourceSpan::new(0.into(), file_content.len().into());
                                collected_errors.push(AppError::YamlParseError {
                                    path: input_path.clone(),
                                    message: "Internal error: Failed to convert parsed YAML to JSON for validation".to_string(),
                                    span: err_span,
                                    source_code: file_content.clone(),
                                });
                                continue;
                            }
                        };
                        let validation_result = compiled_schema.validate(&json_value_for_validation);
                        if let Err(validation_error) = validation_result {
                            // In jsonschema 0.30.0, ValidationError has basic fields but doesn't iterate
                            // Let's just report the single error from the validation failure
                            let fallback_span = SourceSpan::new(0.into(), file_content.len().into());
                            let error_json_path = validation_error.instance_path.to_string();
                            let kind_str = format!("{:?}", validation_error.kind);
                            collected_errors.push(AppError::SchemaValidationError {
                                path: input_path.clone(),
                                message: "Schema validation failed".to_string(),
                                source_code: file_content.clone(),
                                error_span: fallback_span,
                                label_message: format!("Field `{}`: {}", error_json_path, kind_str),
                                instance_path: error_json_path,
                                kind: kind_str,
                            });
                        }
                    }
                    Err(e) => {
                        if let Some(location) = e.location() {
                            let mut offset = 0;
                            for (i, line_content) in file_content.lines().enumerate() {
                                if i < location.line() -1 { offset += line_content.len() + 1; } else { break; }
                            }
                            offset += location.column() -1;
                            let err_span = SourceSpan::new(offset.into(), 1usize.into());
                            collected_errors.push(AppError::YamlParseError {
                                path: input_path.clone(), message: e.to_string(), span: err_span, source_code: file_content.clone(),
                            });
                        } else {
                            let err_span = SourceSpan::new(0.into(), file_content.len().into());
                            collected_errors.push(AppError::YamlParseError {
                                path: input_path.clone(), message: format!("YAML parsing error: {}", e), span: err_span, source_code: file_content.clone(),
                            });
                        }
                    }
                }
            }
            Some("json") => {
                match json_spanned_value::from_str::<SpannedJsonValue>(&file_content) {
                    Ok(spanned_json_doc) => {
                        // For json-spanned-value, we need to convert the spanned value to a regular JsonValue
                        // Let's use the simpler approach of re-parsing the JSON string
                        let plain_json_value: JsonValue = match serde_json::from_str(&file_content) {
                            Ok(val) => val,
                            Err(e) => {
                                collected_errors.push(AppError::JsonParseError {
                                    path: input_path.clone(),
                                    message: "Failed to parse JSON for validation".to_string(),
                                    span: SourceSpan::new(0.into(), file_content.len().into()),
                                    source_code: file_content.clone(),
                                    source: e,
                                });
                                continue;
                            }
                        };
                        
                        let validation_result = compiled_schema.validate(&plain_json_value);
                        if let Err(validation_error) = validation_result {
                            let error_json_path = validation_error.instance_path.to_string();
                            let target_jspan = find_span_for_json_path(&spanned_json_doc, &error_json_path);
                            let target_miette_span = target_jspan.map(|s| convert_json_span(s))
                                .unwrap_or_else(|| SourceSpan::new(0.into(), file_content.len().into()));
                            let kind_str = format!("{:?}", validation_error.kind);
                            collected_errors.push(AppError::SchemaValidationError {
                                path: input_path.clone(),
                                message: "Schema validation failed".to_string(),
                                source_code: file_content.clone(),
                                error_span: target_miette_span,
                                label_message: format!("Field `{}`: {}", error_json_path, kind_str),
                                instance_path: error_json_path,
                                kind: kind_str,
                            });
                        }
                    }
                    Err(e) => {
                        let line = e.line(); let column = e.column(); let mut offset = 0;
                        for (i, line_content) in file_content.lines().enumerate() {
                            if i < line - 1 { offset += line_content.len() + 1; } else { break; }
                        }
                        offset += column - 1;
                        let err_span = SourceSpan::new(offset.into(), 1usize.into());
                        collected_errors.push(AppError::JsonParseError {
                            path: input_path.clone(), 
                            message: e.to_string(), 
                            span: err_span, 
                            source_code: file_content.clone(),
                            source: e,
                        });
                    }
                }
            }
            Some("toml") => {
                println!("Detected TOML file: {:?}", input_path);
                match file_content.parse::<DocumentMut>() {
                    Ok(toml_doc) => {
                        println!("TOML content parsed into DocumentMut successfully.");
                        // Convert DocumentMut to serde_json::Value for validation
                        // Use to_string() and re-parse approach since toml_doc.root is private
                        let toml_as_string = toml_doc.to_string();
                        let json_value_for_validation: JsonValue = match toml::from_str::<toml::Value>(&toml_as_string) {
                            Ok(toml_value) => match serde_json::to_value(toml_value) {
                                Ok(json_val) => json_val,
                                Err(_) => {
                                    let err_span = SourceSpan::new(0.into(), file_content.len().into());
                                    collected_errors.push(AppError::TomlParseError {
                                        path: input_path.clone(),
                                        message: "Internal error: Failed to convert TOML to JSON for validation".to_string(),
                                        span: err_span,
                                        source_code: file_content.clone(),
                                    });
                                    continue;
                                }
                            },
                            Err(_) => {
                                let err_span = SourceSpan::new(0.into(), file_content.len().into());
                                collected_errors.push(AppError::TomlParseError {
                                    path: input_path.clone(),
                                    message: "Internal error: Failed to re-parse TOML string for validation".to_string(),
                                    span: err_span,
                                    source_code: file_content.clone(),
                                });
                                continue;
                            }
                        };
                        let validation_result = compiled_schema.validate(&json_value_for_validation);
                        if let Err(validation_error) = validation_result {
                            let error_json_path = validation_error.instance_path.to_string();
                            let target_toml_span_range = find_span_for_toml_path(toml_doc.as_item(), &error_json_path);
                            let target_miette_span = convert_toml_edit_span(target_toml_span_range)
                                .unwrap_or_else(|| SourceSpan::new(0.into(), file_content.len().into()));

                            let kind_str = format!("{:?}", validation_error.kind);
                            let label_msg = if error_json_path.is_empty() || error_json_path == "/" {
                                format!("Validation failed at root: {}", kind_str)
                            } else {
                                format!("Field `{}`: {}", error_json_path, kind_str)
                            };

                            collected_errors.push(AppError::SchemaValidationError {
                                path: input_path.clone(),
                                message: "Schema validation failed".to_string(),
                                source_code: file_content.clone(),
                                error_span: target_miette_span,
                                label_message: label_msg,
                                instance_path: error_json_path,
                                kind: kind_str,
                            });
                        } else {
                            println!("File {:?} is valid against the schema.", input_path);
                        }
                    }
                    Err(e) => {
                        // Error from parsing into DocumentMut (toml_edit::TomlError)
                        // toml_edit::TomlError has a span() method returning Option<(usize, usize)>
                        collected_errors.push(AppError::TomlParseError {
                            path: input_path.clone(),
                            message: e.message().to_string(),
                            span: e.span().map(|range| { // Use range here
                                let length = if range.end > range.start { range.end - range.start } else { 1 };
                                SourceSpan::new(range.start.into(), length.into())
                            }).unwrap_or_else(|| SourceSpan::new(0.into(), file_content.len().into())),
                            source_code: file_content.clone(),
                        });
                    }
                }
            }
            Some("hcl") => {
                // HCL parsing using the hcl-rs API
                match hcl::from_str::<JsonValue>(&file_content) {
                    Ok(hcl_json_value_for_validation) => {
                        let validation_result = compiled_schema.validate(&hcl_json_value_for_validation);
                        if let Err(validation_error) = validation_result {
                            let fallback_span = SourceSpan::new(0.into(), file_content.len().into());
                            let error_json_path = validation_error.instance_path.to_string();
                            let kind_str = format!("{:?}", validation_error.kind);
                            collected_errors.push(AppError::SchemaValidationError {
                                path: input_path.clone(), 
                                message: "Schema validation failed".to_string(),
                                source_code: file_content.clone(), 
                                error_span: fallback_span,
                                label_message: format!("Field `{}`: {}", error_json_path, kind_str),
                                instance_path: error_json_path, 
                                kind: kind_str,
                            });
                        }
                    }
                    Err(e) => {
                        let err_span = SourceSpan::new(0.into(), file_content.len().into());
                        collected_errors.push(AppError::HclParseError {
                            path: input_path.clone(), 
                            message: format!("HCL parsing failed: {}", e), 
                            span: err_span, 
                            source_code: file_content.clone(),
                        });
                    }
                }
            }
            Some(ext) => {
                println!("Skipping unsupported file type ({}): {:?}", ext, input_path);
            }
            None => {
                println!("Skipping file without extension: {:?}", input_path);
            }
        }
    }
    Ok(collected_errors)
} 