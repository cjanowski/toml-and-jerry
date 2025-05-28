use clap::{Parser, Subcommand};
use miette::Result;
use std::path::PathBuf;

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

fn main() -> Result<()> {
    
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Check {
            inputs,
            schema,
            format,
        } => {
            // TODO: implement validation pipeline
            println!(
                "Would validate {:?} using schema {:?} (output: {})",
                inputs, schema, format
            );
        }
        Cmd::Scaffold { crate_path, out } => {
            // TODO: implement schema generation
            println!(
                "Would scaffold schema from crate {:?} into file {:?}",
                crate_path, out
            );
        }
    }

    Ok(())
}
