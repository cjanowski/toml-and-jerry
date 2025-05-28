use clap::{Parser, Subcommand};
use miette::Result;
use std::path::PathBuf;
// SARIF imports temporarily disabled - will re-implement later
// use serde_sarif::sarif::{
//     ArtifactLocation, Location, Message, PhysicalLocation,
//     Region, Result as SarifResult, Run, Sarif, Tool,
//     ToolComponent,
// };

mod error;
mod schema;
use schema::load_and_compile_schema;
mod validation;
use validation::validate_inputs;
use error::AppError;

#[derive(Parser)]
#[command(
    name = "toml-and-jerry",
    version,
    about = "Validate JSON/TOML/YAML/HCL configs against a JSON Schema or OpenAPI component schema"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Validate config files against a schema
    Check {
        /// Path(s) or glob
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// JSON Schema file (local or URL) or OpenAPI spec
        #[arg(short, long)]
        schema: PathBuf,

        /// Output format: human | json | sarif
        #[arg(long, default_value = "human")]
        format: String,
    },

    /// Generate a starter JSON Schema from Rust types
    Scaffold {
        /// Path to a Rust crate exposing config structs
        #[arg(default_value = ".")]
        crate_path: PathBuf,

        /// File to write the generated schema to
        #[arg(long)]
        out: PathBuf,
    },
}

fn errors_to_sarif(_errors: &[AppError]) -> Result<String, Box<dyn std::error::Error>> {
    // SARIF implementation temporarily disabled - will re-implement later
    // For now, return a minimal valid SARIF structure
    let minimal_sarif = serde_json::json!({
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "toml-and-jerry",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/coryjanowski/toml-and-jerry"
                }
            },
            "results": []
        }]
    });
    
    Ok(serde_json::to_string_pretty(&minimal_sarif)?)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut has_errors = false;

    match cli.cmd {
        Cmd::Check {
            inputs,
            schema,
            format,
        } => {
            let compiled_schema = match load_and_compile_schema(&schema) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{:?}", miette::Report::new(e));
                    std::process::exit(1);
                }
            };
            println!("Validating inputs against schema {:?} (output format: {})", schema, format);
            println!("Schema loaded and compiled successfully.");

            match validate_inputs(inputs, &compiled_schema) {
                Ok(collected_errors) => {
                    if !collected_errors.is_empty() {
                        has_errors = true;
                        match format.as_str() {
                            "json" => {
                                let printable_errors: Vec<validation::PrintableError> = collected_errors
                                    .iter()
                                    .map(|e| e.into())
                                    .collect();
                                match serde_json::to_string_pretty(&printable_errors) {
                                    Ok(json_output) => println!("{}", json_output),
                                    Err(e) => {
                                        eprintln!("Failed to serialize errors to JSON: {}", e);
                                        for err in collected_errors {
                                            eprintln!("{:?}", miette::Report::new(err));
                                        }
                                    }
                                }
                            }
                            "sarif" => {
                                match errors_to_sarif(&collected_errors) {
                                    Ok(sarif_output) => println!("{}", sarif_output),
                                    Err(e) => {
                                        eprintln!("Failed to generate SARIF output: {}", e);
                                        for err in collected_errors {
                                            eprintln!("{:?}", miette::Report::new(err));
                                        }
                                    }
                                }
                            }
                            _ => {
                                println!("\n--- Validation Summary ---");
                                for err in collected_errors {
                                    eprintln!("{:?}", miette::Report::new(err));
                                }
                            }
                        }
                    } else {
                        if format == "json" {
                            println!("[]");
                        } else if format == "sarif" {
                            // Empty SARIF report for no errors
                            match errors_to_sarif(&[]) {
                                Ok(sarif_output) => println!("{}", sarif_output),
                                Err(e) => eprintln!("Failed to generate SARIF output: {}", e),
                            }
                        } else {
                            println!("\n--- Validation Summary ---");
                            println!("All processed files are valid!");
                        }
                    }
                }
                Err(fatal_err) => {
                    eprintln!("{:?}", miette::Report::new(fatal_err));
                    has_errors = true;
                }
            }
        }
        Cmd::Scaffold { crate_path, out } => {
            println!(
                "Would scaffold schema from crate {:?} into file {:?}",
                crate_path, out
            );
        }
    }

    if has_errors {
        std::process::exit(1);
    }
    Ok(())
}
