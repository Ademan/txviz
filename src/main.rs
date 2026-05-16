mod model;

use std::collections::{BTreeSet, HashMap};
use std::{error::Error, fmt, fs, io::Write, path::PathBuf};

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
    #[arg(required = true)]
    input_files: Vec<PathBuf>,
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output_file: PathBuf,
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
    ParseDomain(String),
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
            Self::ParseDomain(msg) => write!(f, "failed to parse txviz document: {msg}"),
            Self::CreateOutput { path, source } => {
                write!(f, "failed to create output {}: {source}", path.display())
            }
            Self::WriteOutput { path, source } => {
                write!(f, "failed to write output {}: {source}", path.display())
            }
        }
    }
}
impl Error for CliError {}

#[derive(Clone)]
struct RenderConfig {
    tx_min_width: f32,
    tx_max_width: f32,
    tx_title_gap: f32,
    io_top_margin: f32,
    io_bottom_margin: f32,
    input_row_gap: f32,
    output_row_gap: f32,
    output_pad_x: f32,
    output_pad_y: f32,
    io_h_margin: f32,
    output_h_margin: f32,
    tx_title_max_lines: usize,
    tx_horizontal_title_padding: f32,
    input_inset_left: f32,
    output_inset_right: f32,
    tx_gap_y: f32,
    line_height: f32,
}
impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            tx_min_width: 320.0,
            tx_max_width: 700.0,
            tx_title_gap: 6.0,
            io_top_margin: 10.0,
            io_bottom_margin: 10.0,
            input_row_gap: 8.0,
            output_row_gap: 8.0,
            output_pad_x: 8.0,
            output_pad_y: 6.0,
            io_h_margin: 40.0,
            output_h_margin: 16.0,
            tx_title_max_lines: 3,
            tx_horizontal_title_padding: 16.0,
            input_inset_left: 16.0,
            output_inset_right: 16.0,
            tx_gap_y: 32.0,
            line_height: 16.0,
        }
    }
}

#[derive(Clone)]
struct TxDoc {
    id: String,
    creation_index: usize,
    title: Option<String>,
    version: Option<i64>,
    locktime: Option<Value>,
    inputs: Vec<InputDoc>,
    outputs: Vec<OutputDoc>,
}
#[derive(Clone)]
struct InputDoc {
    spends: Option<SpendRef>,
}
#[derive(Clone)]
struct OutputDoc {
    out_uid: String,
    creation_index: usize,
    title: String,
    value: Option<String>,
}

#[derive(Clone)]
struct SpendRef {
    tx_ref: String,
    vout: usize,
}

fn main() -> Result<(), CliError> {
    let cli = Cli::parse();
    let request = build_request(cli)?;
    write_output(&request)?;
    println!(
        "Rendered {} document to {} from {} input file(s).",
        match request.format {
            OutputFormat::Html => "HTML",
            OutputFormat::Svg => "SVG",
        },
        request.output_file.display(),
        request.inputs.len()
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
fn write_output(request: &OperationRequest) -> Result<(), CliError> {
    let mut output =
        fs::File::create(&request.output_file).map_err(|source| CliError::CreateOutput {
            path: request.output_file.clone(),
            source,
        })?;
    let generator = format_generator_label(&request.inputs);
    let txs = parse_transactions(request).map_err(CliError::ParseDomain)?;
    let wr = match request.format {
        OutputFormat::Html => emit_html(&mut output, &generator, &txs),
        OutputFormat::Svg => emit_svg(&mut output, &generator, &txs),
    };
    wr.map_err(|source| CliError::WriteOutput {
        path: request.output_file.clone(),
        source,
    })
}

fn parse_transactions(request: &OperationRequest) -> Result<Vec<TxDoc>, String> {
    let mut txs = Vec::new();
    let mut tx_counter = 0usize;
    let mut out_counter = 0usize;
    for doc in &request.inputs {
        let arr = doc
            .payload
            .get("transactions")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{} missing 'transactions' array", doc.source_path.display()))?;
        for tx in arr {
            let tx_id = tx
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| format!("tx_{tx_counter}"));
            let title = tx
                .get("annotations")
                .and_then(|a| a.get("title"))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .or_else(|| tx.get("title").and_then(Value::as_str).map(str::to_owned));
            let version = tx.get("version").and_then(Value::as_i64);
            let locktime = tx.get("locktime").cloned();
            let inputs = tx
                .get("inputs")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .map(|i| {
                            let spends = i
                                .get("spends")
                                .and_then(Value::as_str)
                                .and_then(parse_spend_ref);
                            InputDoc { spends }
                        })
                        .collect()
                })
                .unwrap_or_default();
            let outputs = tx
                .get("outputs")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .enumerate()
                        .map(|(i, o)| {
                            let title = o
                                .get("annotations")
                                .and_then(|a| a.get("title"))
                                .and_then(Value::as_str)
                                .map(str::to_owned)
                                .or_else(|| {
                                    o.get("title").and_then(Value::as_str).map(str::to_owned)
                                })
                                .unwrap_or_else(|| format!("output {i}"));
                            let value = o
                                .get("amount_sat")
                                .and_then(Value::as_u64)
                                .map(|v| format!("{v} sat"))
                                .or_else(|| {
                                    o.get("amount_expr")
                                        .and_then(Value::as_str)
                                        .map(str::to_owned)
                                });
                            let out_uid = format!("{tx_id}:{i}");
                            let creation_index = out_counter;
                            out_counter += 1;
                            OutputDoc {
                                out_uid,
                                creation_index,
                                title,
                                value,
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();
            txs.push(TxDoc {
                id: tx_id,
                creation_index: tx_counter,
                title,
                version,
                locktime,
                inputs,
                outputs,
            });
            tx_counter += 1;
        }
    }
    Ok(txs)
}

fn parse_spend_ref(raw: &str) -> Option<SpendRef> {
    let (tx_ref, vout_raw) = raw.rsplit_once(':')?;
    let vout = vout_raw.parse::<usize>().ok()?;
    Some(SpendRef {
        tx_ref: tx_ref.to_owned(),
        vout,
    })
}

struct TextMeasurer;
impl TextMeasurer {
    fn width(&self, s: &str) -> f32 {
        s.chars().count() as f32 * 8.0
    }
}

fn emit_html(mut writer: impl Write, generator: &str, txs: &[TxDoc]) -> std::io::Result<()> {
    let mut svg = Vec::new();
    emit_svg(&mut svg, generator, txs)?;
    let escaped = html_escape::encode_text(generator);
    write!(
        writer,
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"generator\" content=\"{}\"><title>txviz</title></head><body>{}</body></html>",
        escaped,
        String::from_utf8_lossy(&svg)
    )
}
fn emit_svg(mut writer: impl Write, generator: &str, txs: &[TxDoc]) -> std::io::Result<()> {
    let cfg = RenderConfig::default();
    let m = TextMeasurer;
    let columns = build_layout_columns(txs).unwrap_or_else(|_| vec![txs.iter().collect()]);
    let mut max_y: f32 = 0.0;
    let mut groups = String::new();
    let mut links = String::new();
    let mut max_w: f32 = 0.0;
    let mut output_anchors: HashMap<(String, usize), (f32, f32)> = HashMap::new();
    let mut input_anchors: Vec<(Option<SpendRef>, f32, f32)> = Vec::new();
    for (column_idx, col) in columns.iter().enumerate() {
        let mut y = 20.0;
        let x = 20.0 + column_idx as f32 * (cfg.tx_max_width + 120.0);
        for tx in col {
            let (g, h, w, in_anchors, out_anchors) = layout_tx(tx, &cfg, &m, x, y);
            y += h + cfg.tx_gap_y;
            max_w = max_w.max(x + w + 40.0);
            groups.push_str(&g);
            for (idx, (ox, oy)) in out_anchors.into_iter().enumerate() {
                output_anchors.insert((tx.id.clone(), idx), (ox, oy));
            }
            for (idx, (ix, iy)) in in_anchors.into_iter().enumerate() {
                input_anchors.push((tx.inputs[idx].spends.clone(), ix, iy));
            }
        }
        max_y = max_y.max(y);
    }
    links.push_str(
        "  <g class=\"spend-links\" fill=\"none\" stroke=\"black\" stroke-width=\"1\">\n",
    );
    for (spends, input_left, input_y) in input_anchors {
        if let Some(spend_ref) = spends {
            if let Some((output_right, output_y)) =
                output_anchors.get(&(spend_ref.tx_ref, spend_ref.vout))
            {
                let separation = input_left - output_right;
                let control_offset = separation / 3.0;
                let c1x = output_right + control_offset;
                let c2x = input_left - control_offset;
                links.push_str(&format!(
                    "    <path class=\"spend-link\" d=\"M {} {} C {} {}, {} {}, {} {}\"/>\n",
                    output_right, output_y, c1x, output_y, c2x, input_y, input_left, input_y
                ));
            }
        }
    }
    links.push_str("  </g>\n");
    write!(
        writer,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{}\" height=\"{}\">\n  <metadata>{}</metadata>\n{}{}\n</svg>\n",
        max_w.max(200.0),
        max_y.max(120.0),
        html_escape::encode_text(generator),
        links,
        groups
    )
}

fn layout_tx(
    tx: &TxDoc,
    c: &RenderConfig,
    m: &TextMeasurer,
    left: f32,
    top: f32,
) -> (String, f32, f32, Vec<(f32, f32)>, Vec<(f32, f32)>) {
    let header = if tx.version.is_some() || tx.locktime.is_some() {
        c.line_height
    } else {
        0.0
    };
    let input_w = 90.0;
    let input_h = 24.0;
    let out_h = c.output_pad_y + c.line_height + c.output_pad_y;
    let mut out_w: f32 = 0.0;
    for o in &tx.outputs {
        let tw = m.width(&o.title);
        let vw = o.value.as_ref().map(|v| m.width(v)).unwrap_or(0.0);
        let content = if o.value.is_some() {
            tw + c.output_h_margin + vw
        } else {
            tw
        };
        out_w = out_w.max(c.output_pad_x + content + c.output_pad_x);
    }
    let min_internal =
        c.input_inset_left + input_w + c.io_h_margin + out_w.max(60.0) + c.output_inset_right;
    let title_w = tx.title.as_ref().map(|t| m.width(t)).unwrap_or(0.0);
    let mut tx_w = (title_w + 2.0 * c.tx_horizontal_title_padding)
        .clamp(c.tx_min_width, c.tx_max_width)
        .max(min_internal);
    tx_w = tx_w.min(c.tx_max_width);
    let io_y = top + header.max(c.io_top_margin);
    let in_last = if tx.inputs.is_empty() {
        io_y
    } else {
        io_y + tx.inputs.len() as f32 * input_h
            + (tx.inputs.len().saturating_sub(1) as f32) * c.input_row_gap
    };
    let out_last = if tx.outputs.is_empty() {
        io_y
    } else {
        io_y + tx.outputs.len() as f32 * out_h
            + (tx.outputs.len().saturating_sub(1) as f32) * c.output_row_gap
    };
    let tx_h = in_last.max(out_last) - top + c.io_bottom_margin;
    let right = left + tx_w;
    let mut s = String::new();
    s.push_str("  <g class=\"tx\">\n");
    if let Some(t) = &tx.title {
        let ty = top - c.tx_title_gap;
        s.push_str(&format!("    <g class=\"tx-title\"><text x=\"{}\" y=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\">{}</text></g>\n",left,ty,html_escape::encode_text(t)));
    }
    s.push_str("    <g class=\"tx-body\">\n");
    s.push_str(&format!("      <rect class=\"tx-box\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\"/>\n",left,top,tx_w,tx_h));
    if header > 0.0 {
        let hdr = format!(
            "v={} locktime={}",
            tx.version
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            tx.locktime
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string())
        );
        s.push_str(&format!("      <g class=\"tx-header\"><text x=\"{}\" y=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\">{}</text></g>\n",left+12.0,top+14.0,html_escape::encode_text(&hdr)));
    }
    s.push_str("      <g class=\"tx-inputs\">\n");
    let mut input_anchors = Vec::new();
    for i in 0..tx.inputs.len() {
        let y = io_y + i as f32 * (input_h + c.input_row_gap);
        let input_left = left + c.input_inset_left;
        s.push_str(&format!("        <g><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\"/></g>\n",input_left,y,input_w,input_h));
        input_anchors.push((input_left, y + input_h / 2.0));
    }
    s.push_str("      </g>\n");
    s.push_str("      <g class=\"tx-outputs\">\n");
    let out_left = right - c.output_inset_right - out_w.max(60.0);
    let mut output_anchors = Vec::new();
    for (j, o) in tx.outputs.iter().enumerate() {
        let y = io_y + j as f32 * (out_h + c.output_row_gap);
        let ow = out_w.max(60.0);
        s.push_str(&format!("        <g data-title=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"1\"/><text x=\"{}\" y=\"{}\" text-anchor=\"start\" fill=\"none\" stroke=\"black\" stroke-width=\"1\">{}</text>",html_escape::encode_double_quoted_attribute(&o.title),out_left,y,ow,out_h,out_left+c.output_pad_x,y+c.output_pad_y+c.line_height-3.0,html_escape::encode_text(&o.title)));
        if let Some(v) = &o.value {
            s.push_str(&format!("<text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"none\" stroke=\"black\" stroke-width=\"1\">{}</text>",out_left+ow-c.output_pad_x,y+c.output_pad_y+c.line_height-3.0,html_escape::encode_text(v)));
        }
        s.push_str("</g>\n");
        output_anchors.push((out_left + ow, y + out_h / 2.0));
    }
    s.push_str("      </g>\n    </g>\n  </g>\n");
    (s, tx_h, tx_w, input_anchors, output_anchors)
}

fn build_layout_columns<'a>(txs: &'a [TxDoc]) -> Result<Vec<Vec<&'a TxDoc>>, String> {
    let id_to_tx: HashMap<&str, &TxDoc> = txs.iter().map(|t| (t.id.as_str(), t)).collect();
    let output_creation: HashMap<(String, usize), usize> = txs
        .iter()
        .flat_map(|tx| {
            tx.outputs
                .iter()
                .enumerate()
                .map(move |(vout, out)| ((tx.id.clone(), vout), out.creation_index))
        })
        .collect();

    let mut remaining: BTreeSet<&str> = txs.iter().map(|t| t.id.as_str()).collect();
    let mut laid_out: BTreeSet<&str> = BTreeSet::new();
    let mut columns: Vec<Vec<&TxDoc>> = Vec::new();

    let mut no_input: Vec<&TxDoc> = txs
        .iter()
        .filter(|tx| {
            !tx.inputs.iter().any(|i| {
                i.spends
                    .as_ref()
                    .is_some_and(|s| id_to_tx.contains_key(s.tx_ref.as_str()))
            })
        })
        .collect();
    no_input.sort_by_key(|t| t.creation_index);
    for tx in &no_input {
        remaining.remove(tx.id.as_str());
        laid_out.insert(tx.id.as_str());
    }
    if !no_input.is_empty() {
        columns.push(no_input);
    }

    while !remaining.is_empty() {
        let mut candidate_ids: BTreeSet<&str> = BTreeSet::new();
        for parent_id in &laid_out {
            if let Some(parent) = id_to_tx.get(parent_id) {
                for out in &parent.outputs {
                    for tx in txs.iter().filter(|tx| {
                        tx.inputs.iter().any(|i| {
                            i.spends
                                .as_ref()
                                .is_some_and(|s| format!("{}:{}", s.tx_ref, s.vout) == out.out_uid)
                        })
                    }) {
                        if remaining.contains(tx.id.as_str()) {
                            candidate_ids.insert(tx.id.as_str());
                        }
                    }
                }
            }
        }
        let mut next: Vec<&TxDoc> = candidate_ids
            .into_iter()
            .filter_map(|id| id_to_tx.get(id).copied())
            .filter(|tx| {
                tx.inputs.iter().all(|i| {
                    i.spends.as_ref().is_none_or(|s| {
                        !id_to_tx.contains_key(s.tx_ref.as_str())
                            || laid_out.contains(s.tx_ref.as_str())
                    })
                })
            })
            .collect();

        if next.is_empty() {
            return Err("unsatisfied transaction dependencies".to_string());
        }

        let tx_row: HashMap<&str, usize> = columns
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, tx)| (tx.id.as_str(), i))
            .collect();
        next.sort_by_key(|tx| {
            let spends_tx_key = tx
                .inputs
                .iter()
                .filter_map(|i| i.spends.as_ref())
                .filter_map(|s| tx_row.get(s.tx_ref.as_str()).copied())
                .min()
                .unwrap_or(usize::MAX);
            let spends_out_key = tx
                .inputs
                .iter()
                .filter_map(|i| i.spends.as_ref())
                .filter_map(|s| output_creation.get(&(s.tx_ref.clone(), s.vout)).copied())
                .min()
                .unwrap_or(usize::MAX);
            (spends_tx_key, spends_out_key, tx.creation_index)
        });

        for tx in &next {
            remaining.remove(tx.id.as_str());
            laid_out.insert(tx.id.as_str());
        }
        columns.push(next);
    }

    Ok(columns)
}

fn format_generator_label(inputs: &[InputDocument]) -> String {
    let p = inputs
        .first()
        .map(|i| i.source_path.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    format!("txviz ({p})")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn svg_contains_tx_group() {
        let tx = TxDoc {
            id: "a".into(),
            creation_index: 0,
            title: Some("A title".into()),
            version: Some(2),
            locktime: None,
            inputs: vec![InputDoc { spends: None }],
            outputs: vec![OutputDoc {
                out_uid: "a:0".into(),
                creation_index: 0,
                title: "out".into(),
                value: Some("1 sat".into()),
            }],
        };
        let mut out = Vec::new();
        emit_svg(&mut out, "txviz", &[tx]).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("class=\"tx\""));
        assert!(s.contains("class=\"tx-outputs\""));
    }
    #[test]
    fn top_margin_collapse_uses_max() {
        let cfg = RenderConfig::default();
        let tx = TxDoc {
            id: "a".into(),
            creation_index: 0,
            title: None,
            version: Some(1),
            locktime: None,
            inputs: vec![InputDoc { spends: None }],
            outputs: vec![],
        };
        let (_svg, h, _w) = layout_tx(&tx, &cfg, &TextMeasurer, 20.0, 20.0);
        assert!(h > 0.0);
    }
}
