use miette::{Diagnostic, SourceSpan};
use std::path::PathBuf;
use thiserror::Error;
// No need to import SpannedJsonValue or JsonSpan here if they are only used in main.rs for now
// unless AppError variants themselves need to hold them directly, which they don't currently.

#[derive(Debug, Error, Diagnostic)]
pub enum AppError { // Made AppError public
    #[error("Failed to read file {path:?}: {source}")]
    #[diagnostic(code(app::io::read_file))]
    FileReadError {
        path: PathBuf,
        #[source] source: std::io::Error,
        #[label("while reading this file")] span: Option<SourceSpan> 
    },

    #[error("Failed to fetch schema from URL {url}: {source}")]
    #[diagnostic(code(app::network::fetch_schema))]
    SchemaFetchError {
        url: String,
        #[source] source: reqwest::Error,
    },

    #[error("Failed to parse schema (from {source_display:?}): {source}")]
    #[diagnostic(code(app::schema::parse_error))]
    SchemaParseError {
        source_display: String, 
        #[source] source: serde_json::Error,
    },

    #[error("Failed to compile schema (from {source_display:?}): {source}")]
    #[diagnostic(code(app::schema::compile_error))]
    SchemaCompileError {
        source_display: String, 
        #[source] source: jsonschema::ValidationError<'static>,
    },

    #[error("YAML parsing error in file {path:?}: {message}")]
    #[diagnostic(code(app::yaml::parse_error))]
    YamlParseError {
        path: PathBuf,
        message: String,
        #[label = "{message}"]
        span: SourceSpan,
        #[source_code]
        source_code: String,
    },

    #[error("Schema validation error in file {path:?}: {message}")]
    #[diagnostic(code(app::schema::validation_error))]
    SchemaValidationError {
        path: PathBuf,
        message: String, 
        #[source_code]
        source_code: String, 
        #[label("{label_message}")]
        error_span: SourceSpan,
        label_message: String, 
        instance_path: String,
        kind: String, 
    },

    #[error("JSON parsing error in file {path:?}: {message}")]
    #[diagnostic(code(app::json::parse_error))]
    JsonParseError {
        path: PathBuf,
        message: String,
        #[label = "{message}"]
        span: SourceSpan,
        #[source_code]
        source_code: String,
        #[source] source: serde_json::Error,
    },

    #[error("TOML parsing error in file {path:?}: {message}")]
    #[diagnostic(code(app::toml::parse_error))]
    TomlParseError {
        path: PathBuf,
        message: String,
        #[label = "{message}"]
        span: SourceSpan,
        #[source_code]
        source_code: String,
    },

    #[error("HCL parsing error in file {path:?}: {message}")]
    #[diagnostic(code(app::hcl::parse_error))]
    HclParseError {
        path: PathBuf,
        message: String,
        #[label = "{message}"]
        span: SourceSpan, 
        #[source_code]
        source_code: String,
    },

    #[error("Invalid schema path: {path_display}")]
    #[diagnostic(code(app::schema::invalid_path))]
    InvalidSchemaPath {
        path_display: String,
    },
}


 