//! # slf-dump
//!
//! A command-line tool to inspect and dump the contents of Selafin binary files.
//
use std::io;
use std::process::ExitCode;
use std::str::FromStr;
use std::{process, vec};

use clap::Parser;
use serde_json::json;
use serde_json::value::Value as JsonValue;

use free_surface::aui::configviewer::{create_config_viewer, ConfigViewer, ConfigViewerOptions};

use free_surface::aui::Format;
use free_surface::storage::selafin::{parse_file, Selafin};

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(
    name = "slf-dump",
    version,
    about = "Dump the contents of a Selafin (.slf) file"
)]
struct Args {
    /// Path to the Selafin file
    file: std::path::PathBuf,

    /// Comma-separated list of sections to display.
    ///
    /// Supported tokens: title, npoints, nelements, nlayers, nplanes,
    /// variables, variables+units, points, points:layer=N,
    /// elements, elements:layer=N, datetime, results
    #[arg(long, value_delimiter = ',')]
    show: Vec<ShowToken>,

    /// Print values of variable NAME at time-step index T (0-based).
    /// Format: NAME:T  - repeatable, e.g. --history DEPTH:0 --history DEPTH:1
    #[arg(long, value_name = "NAME:T")]
    history: Vec<HistoryQuery>,

    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Human)]
    format: Format,

    /// Compact JSON output (no indentation). Only meaningful with --format json.
    #[arg(long, conflicts_with = "pretty")]
    compact: bool,

    /// Pretty-printed JSON output (indented). Only meaningful with --format json.
    #[arg(long, conflicts_with = "compact", default_value_t = true)]
    pretty: bool,

    /// Colorize the output
    #[arg(long, default_value_t=clap::ColorChoice::Auto)]
    color: clap::ColorChoice,
}

// ---------------------------------------------------------------------------
// Parsed show-token
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum ShowToken {
    Title,
    NPoints,
    NElements,
    NLayers,
    Variables { with_units: bool },
    Points { layer: Option<usize> },
    Elements { layer: Option<usize> },
    Datetime,
    Results,
}

impl FromStr for ShowToken {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        // tokens with parameters: "points:layer=N", "elements:layer=N"
        if let Some(rest) = s.strip_prefix("points:layer=") {
            let n: usize = rest
                .parse()
                .map_err(|_| format!("invalid layer index in '{s}'"))?;
            return Ok(ShowToken::Points { layer: Some(n) });
        }
        if let Some(rest) = s.strip_prefix("elements:layer=") {
            let n: usize = rest
                .parse()
                .map_err(|_| format!("invalid layer index in '{s}'"))?;
            return Ok(ShowToken::Elements { layer: Some(n) });
        }
        match s {
            "title" => Ok(ShowToken::Title),
            "npoints" => Ok(ShowToken::NPoints),
            "nelements" => Ok(ShowToken::NElements),
            "nlayers" | "nplanes" => Ok(ShowToken::NLayers),
            "variables" => Ok(ShowToken::Variables { with_units: false }),
            "variables+units" => Ok(ShowToken::Variables { with_units: true }),
            "points" => Ok(ShowToken::Points { layer: None }),
            "elements" => Ok(ShowToken::Elements { layer: None }),
            "datetime" => Ok(ShowToken::Datetime),
            "results" => Ok(ShowToken::Results),
            other => Err(format!("unknown section '{other}'")),
        }
    }
}
// ---------------------------------------------------------------------------
// Parsing args for rendering
// ---------------------------------------------------------------------------

fn get_config_viewer_options(args: &Args) -> ConfigViewerOptions {
    match args.format {
        Format::Damocles => ConfigViewerOptions::Damocles,
        Format::Human => ConfigViewerOptions::Human { color: args.color },
        Format::Json => ConfigViewerOptions::Json {
            pretty: !args.compact,
        },
        Format::Machine => ConfigViewerOptions::Machine,
    }
}

// ---------------------------------------------------------------------------
// Parsed history query
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct HistoryQuery {
    variable: String,
    time_index: usize,
}

impl FromStr for HistoryQuery {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, t_str) = s
            .rsplit_once(':')
            .ok_or_else(|| format!("history query '{s}' must be in NAME:T format"))?;
        let time_index: usize = t_str
            .parse()
            .map_err(|_| format!("invalid time index '{t_str}' in history query '{s}'"))?;
        Ok(HistoryQuery {
            variable: name.to_string(),
            time_index,
        })
    }
}

// ---------------------------------------------------------------------------
// Section renderers (format-agnostic)
// ---------------------------------------------------------------------------

fn render_title(e: &mut dyn ConfigViewer, slf: &Selafin) {
    //e.emit_section_start("title");
    e.emit_kv("title", &json!(slf.title()));
    //e.emit_section_end();
}

fn render_npoints(e: &mut dyn ConfigViewer, slf: &Selafin) {
    e.emit_section_start("npoints");
    e.emit_kv("npoints", &json!(&slf.geometry().points_count()));
    e.emit_section_end();
}

fn render_nelements(e: &mut dyn ConfigViewer, slf: &Selafin) {
    e.emit_section_start("nelements");
    e.emit_kv("nelements", &json!(&slf.geometry().elements_count()));
    e.emit_section_end();
}

fn render_nlayers(e: &mut dyn ConfigViewer, slf: &Selafin) {
    e.emit_section_start("nlayers");
    e.emit_kv("nlayers", &json!(&slf.geometry().planes_cnt()));
    e.emit_section_end();
}

fn render_variables(e: &mut dyn ConfigViewer, slf: &Selafin, with_units: bool) {
    e.emit_section_start("variables");
    if with_units {
        let headers = ["name", "unit", "kind"];
        let rows: Vec<Vec<JsonValue>> = slf
            .var_defs()
            .iter()
            .map(|v| vec![json!(v.name), json!(v.unit), json!("linear")])
            .chain(
                slf.cld_defs()
                    .iter()
                    .map(|v| vec![json!(v.name), json!(v.unit), json!("quadratic")]),
            )
            .collect();
        e.emit_table("variables", &headers, &rows);
    } else {
        let names: Vec<String> = slf
            .var_defs()
            .iter()
            .chain(slf.cld_defs().iter())
            .map(|v| v.name.clone())
            .collect();
        e.emit_kv("variables", &JsonValue::from(names));
    }
    e.emit_section_end();
}

fn render_points(e: &mut dyn ConfigViewer, slf: &Selafin, layer: Option<usize>) {
    let geo = slf.geometry();
    let section_name = match layer {
        Some(n) => format!("points:layer={n}"),
        None => "points".to_string(),
    };
    e.emit_section_start(&section_name);

    let (xs, ys): (Vec<f64>, Vec<f64>) = geo.points_raw().to_vec();

    let (xs_slice, ys_slice): (&[f64], &[f64]) = match layer {
        None => (&xs, &ys),
        Some(n) => {
            let ppl = geo.points_per_layer();
            let start = n * ppl;
            let end = start + ppl;
            if end > xs.len() {
                eprintln!(
                    "error: layer {n} does not exist (file has {} layers)",
                    geo.planes_cnt()
                );
                process::exit(1);
            }
            (&xs[start..end], &ys[start..end])
        }
    };

    let rows: Vec<Vec<JsonValue>> = xs_slice
        .iter()
        .zip(ys_slice.iter())
        .enumerate()
        .map(|(i, (x, y))| vec![json!(i), json!(x), json!(y)])
        .collect();
    e.emit_table("points", &["index", "x", "y"], &rows);
    e.emit_section_end();
}

fn render_elements(e: &mut dyn ConfigViewer, slf: &Selafin, layer: Option<usize>) {
    let geo = slf.geometry();
    let section_name = match layer {
        Some(n) => format!("elements:layer={n}"),
        None => "elements".to_string(),
    };
    e.emit_section_start(&section_name);

    let npd2 = geo.point_per_layer_element();
    let point_names: Vec<String> = (0..npd2).map(|i| format!("n{i}")).collect();
    let headers: Vec<&str> = std::iter::once("index")
        .chain(point_names.iter().map(String::as_str))
        .collect();

    let layers_to_show: Vec<usize> = match layer {
        Some(n) => {
            if n >= geo.planes_cnt() as usize {
                eprintln!(
                    "error: layer {n} does not exist (file has {} layers)",
                    geo.planes_cnt()
                );
                process::exit(1);
            }
            vec![n]
        }
        None => (0..geo.planes_cnt() as usize).collect(),
    };

    let mut rows: Vec<Vec<JsonValue>> = Vec::new();
    for l in layers_to_show {
        if let Some(slice) = geo.ikle2(l) {
            for (i, chunk) in slice.chunks(npd2).enumerate() {
                let mut row = vec![JsonValue::from(i)];
                row.extend(chunk.iter().map(|&n| JsonValue::from(n)));
                rows.push(row);
            }
        }
    }
    e.emit_table("elements", &headers, &rows);
    e.emit_section_end();
}

fn render_datetime(e: &mut dyn ConfigViewer, slf: &Selafin) {
    e.emit_section_start("datetime");
    match slf.datetime() {
        Some(dt) => e.emit_kv(
            "datetime",
            &JsonValue::from(dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        ),
        None => e.emit_kv("datetime", &JsonValue::Null),
    }
    e.emit_section_end();
}

fn render_results(e: &mut dyn ConfigViewer, slf: &Selafin) {
    let ts = slf.results();
    e.emit_section_start("results");

    e.emit_kv("step_count", &JsonValue::from(ts.step_count()));
    e.emit_kv("var_count", &JsonValue::from(ts.var_count()));
    e.emit_section_end();
}

fn render_history(e: &mut dyn ConfigViewer, slf: &Selafin, query: &HistoryQuery) {
    let ts = slf.results();
    let section_name = format!("history:{}:{}", query.variable, query.time_index);
    e.emit_section_start(&section_name);

    if query.time_index >= ts.step_count() {
        eprintln!(
            "error: time index {} out of range (file has {} steps)",
            query.time_index,
            ts.step_count()
        );
        process::exit(1);
    }

    let ve = match ts.get_var(&query.variable) {
        Some(v) => v,
        None => {
            eprintln!("error: variable '{}' not found in file", query.variable);
            eprintln!("available variables:");
            for (name, _) in ts.iter_vars() {
                eprintln!("  {name}");
            }
            process::exit(1);
        }
    };

    let values = &ve.values[query.time_index];
    let rows: Vec<Vec<JsonValue>> = match values {
        free_surface::storage::selafin::container::SlfArray1D::Float(v) => v
            .iter()
            .enumerate()
            .map(|(i, x)| vec![JsonValue::from(i), JsonValue::from(*x)])
            .collect(),
        free_surface::storage::selafin::container::SlfArray1D::Double(v) => v
            .iter()
            .enumerate()
            .map(|(i, x)| vec![JsonValue::from(i), JsonValue::from(*x)])
            .collect(),
    };
    e.emit_table("values", &["node", "value"], &rows);
    e.emit_section_end();
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> ExitCode {
    let args = Args::parse();

    let tokens: &Vec<ShowToken> = &args.show;
    let history_queries: &Vec<HistoryQuery> = &args.history;

    if tokens.is_empty() && history_queries.is_empty() {
        eprintln!("nothing to show - use --show and/or --history");
        eprintln!("run with --help for usage");
        process::exit(1);
    }

    // Parse the file
    let slf = parse_file(&args.file).unwrap_or_else(|e| {
        eprintln!("error: failed to parse '{}': {e}", args.file.display());
        process::exit(1);
    });

    let stdout = io::stdout();
    let mut renderer = create_config_viewer(stdout.lock(), get_config_viewer_options(&args));

    run_sections(renderer.as_mut(), &slf, tokens, history_queries);
    renderer.finish();

    ExitCode::SUCCESS
}

fn run_sections(
    e: &mut dyn ConfigViewer,
    slf: &Selafin,
    tokens: &[ShowToken],
    history_queries: &[HistoryQuery],
) {
    for token in tokens {
        match token {
            ShowToken::Title => render_title(e, slf),
            ShowToken::NPoints => render_npoints(e, slf),
            ShowToken::NElements => render_nelements(e, slf),
            ShowToken::NLayers => render_nlayers(e, slf),
            ShowToken::Variables { with_units } => render_variables(e, slf, *with_units),
            ShowToken::Points { layer } => render_points(e, slf, *layer),
            ShowToken::Elements { layer } => render_elements(e, slf, *layer),
            ShowToken::Datetime => render_datetime(e, slf),
            ShowToken::Results => render_results(e, slf),
        }
    }
    for query in history_queries {
        render_history(e, slf, query);
    }
}
