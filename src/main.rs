mod model;

use std::{error::Error, fmt, fs, path::PathBuf};

use clap::{Parser, ValueEnum};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Html,
    Svg,
}

#[derive(Debug, Parser)]
#[command(name = "txviz", version, about = "Prepare txviz rendering operations")]
struct Cli {
    /// One or more input JSON files.
    #[arg(required = true)]
    input_files: Vec<PathBuf>,

    /// Target output file path.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output_file: PathBuf,

    /// Desired output format.
    #[arg(short = 'f', long = "format", value_enum, default_value_t = OutputFormat::Svg)]
    format: OutputFormat,
}

#[derive(Debug)]
struct InputDocument {
    source_path: PathBuf,
    payload: Value,
}

#[derive(Debug)]
struct OperationRequest {
    inputs: Vec<InputDocument>,
    output_file: PathBuf,
    format: OutputFormat,
}

#[derive(Debug)]
enum CliError {
    ReadInput {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseInputJson {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadInput { path, source } => {
                write!(f, "failed to read {}: {source}", path.display())
            }
            Self::ParseInputJson { path, source } => {
                write!(f, "failed to parse JSON {}: {source}", path.display())
            }
        }
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadInput { source, .. } => Some(source),
            Self::ParseInputJson { source, .. } => Some(source),
        }
    }
}

fn main() -> Result<(), CliError> {
    let cli = Cli::parse();
    let request = build_request(cli)?;

    println!(
        "Prepared operation with {} input file(s), output {}, format {:?}.",
        request.inputs.len(),
        request.output_file.display(),
        request.format
    );

    Ok(())
}

fn build_request(cli: Cli) -> Result<OperationRequest, CliError> {
    let mut inputs = Vec::with_capacity(cli.input_files.len());

    for input_path in cli.input_files {
        let raw = fs::read_to_string(&input_path).map_err(|source| CliError::ReadInput {
            path: input_path.clone(),
            source,
        })?;

        let payload = serde_json::from_str(&raw).map_err(|source| CliError::ParseInputJson {
            path: input_path.clone(),
            source,
        })?;

        inputs.push(InputDocument {
            source_path: input_path,
            payload,
        });
    }

    Ok(OperationRequest {
        inputs,
        output_file: cli.output_file,
        format: cli.format,
    })
}
