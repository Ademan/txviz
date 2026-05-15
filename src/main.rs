mod model;

use std::{
    error::Error,
    fmt, fs,
    io::Write,
    path::{Path, PathBuf},
};

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
    CreateOutput {
        path: PathBuf,
        source: std::io::Error,
    },
    WriteOutput {
        path: PathBuf,
        source: std::io::Error,
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
            Self::CreateOutput { path, source } => {
                write!(f, "failed to create output {}: {source}", path.display())
            }
            Self::WriteOutput { path, source } => {
                write!(f, "failed to write output {}: {source}", path.display())
            }
        }
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadInput { source, .. } => Some(source),
            Self::ParseInputJson { source, .. } => Some(source),
            Self::CreateOutput { source, .. } => Some(source),
            Self::WriteOutput { source, .. } => Some(source),
        }
    }
}

fn main() -> Result<(), CliError> {
    let cli = Cli::parse();
    let request = build_request(cli)?;
    write_empty_output(&request)?;

    println!(
        "Rendered empty {} document to {} from {} input file(s).",
        match request.format {
            OutputFormat::Html => "HTML",
            OutputFormat::Svg => "SVG",
        },
        request.output_file.display(),
        request.inputs.len(),
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

fn write_empty_output(request: &OperationRequest) -> Result<(), CliError> {
    let mut output =
        fs::File::create(&request.output_file).map_err(|source| CliError::CreateOutput {
            path: request.output_file.clone(),
            source,
        })?;

    let generator = format_generator_label(&request.inputs);
    let write_result = match request.format {
        OutputFormat::Html => emit_empty_html(&mut output, &generator),
        OutputFormat::Svg => emit_empty_svg(&mut output, &generator),
    };

    write_result.map_err(|source| CliError::WriteOutput {
        path: request.output_file.clone(),
        source,
    })
}

fn format_generator_label(inputs: &[InputDocument]) -> String {
    let first_path = inputs
        .first()
        .map(|input| input.source_path.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    // Touch payload so input processing remains meaningful as implementation evolves.
    let _payload_count = inputs
        .iter()
        .filter(|input| !input.payload.is_null())
        .count();
    format!("txviz ({first_path})")
}

fn emit_empty_html(mut writer: impl Write, generator_label: &str) -> std::io::Result<()> {
    let escaped_generator = html_escape::encode_text(generator_label);
    write!(
        writer,
        "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"generator\" content=\"{}\">\n  <title>txviz</title>\n</head>\n<body></body>\n</html>\n",
        escaped_generator
    )
}

fn emit_empty_svg(mut writer: impl Write, generator_label: &str) -> std::io::Result<()> {
    let escaped_generator = html_escape::encode_text(generator_label);
    write!(
        writer,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\">\n  <metadata>{}</metadata>\n</svg>\n",
        escaped_generator
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_generator_in_html_output() {
        let mut out = Vec::new();
        emit_empty_html(&mut out, "bad <&> \"label\"").unwrap();
        let rendered = String::from_utf8(out).unwrap();
        assert!(rendered.contains("bad &lt;&amp;&gt; &quot;label&quot;"));
    }

    #[test]
    fn emits_standards_compliant_empty_svg() {
        let mut out = Vec::new();
        emit_empty_svg(&mut out, "txviz").unwrap();
        let rendered = String::from_utf8(out).unwrap();
        assert!(rendered.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(rendered.contains("<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\">"));
    }
}
