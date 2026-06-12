use core::fmt;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use clap::Parser;

use free_surface::aui::configviewer::{create_config_viewer, ConfigViewer, ConfigViewerOptions};
use free_surface::aui::Format;
use free_surface::config::configvalue::ConfigValue;
use free_surface::config::dicofile::DicoKeyword;
use free_surface::config::{self, dicofile::Dico};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------
#[derive(Clone)]
enum DocInfo {
    Help,
    ChoiceOptions,
    DefaultValue,
    //AlternateKeywordName,
    Type,
    Nargs,
    Boundaries,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Config file (steering file a.k.a "cas" file) to check
    config: String,

    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Damocles)]
    format: Format,

    /// Path of the dico file used to parse the config
    #[arg(long, default_value = "data/dico/telemac2d.dico")]
    dico: PathBuf,

    /// Print the explicitly-set config values after a successful parse
    #[arg(long)]
    dump: bool,

    /// Print all config values (including defaults) after a successful parse
    #[arg(long)]
    full_dump: bool,

    /// Compact JSON output (no indentation). Only meaningful with --format json.
    #[arg(long, conflicts_with = "pretty")]
    compact: bool,

    /// Pretty-printed JSON output (indented). Only meaningful with --format json.
    #[arg(long, conflicts_with = "compact", default_value_t = true)]
    pretty: bool,

    /// Colorize the output
    #[arg(long, default_value_t=clap::ColorChoice::Auto)]
    color: clap::ColorChoice,

    /// Show some documentation alongside each variable
    ///
    /// This only has an influence if --dump or --full-dump is used.
    /// Furthermore, only some formatter (such as Damocles or Porcelain) handles this.
    /// Json formatter does not handle that.
    /// Supported value: help, choice_option, default_value, type, nargs, boundaries
    #[arg(long, value_delimiter = ',')]
    extra_doc: Vec<DocInfo>,
}

impl FromStr for DocInfo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "help" => Ok(DocInfo::Help),
            "choice_options" => Ok(DocInfo::ChoiceOptions),
            "default_value" => Ok(DocInfo::DefaultValue),
            "type" => Ok(DocInfo::Type),
            "nargs" => Ok(DocInfo::Nargs),
            "boundaries" => Ok(DocInfo::Boundaries),
            other => Err(format!("unknown extra-info '{other}'")),
        }
    }
}
// ---------------------------------------------------------------------------
// Error management helpers
// ---------------------------------------------------------------------------

/// A thin wrapper so we can collect heterogeneous errors into one list.
type Errors = Vec<Box<dyn std::error::Error>>;

fn one_err(e: impl std::error::Error + 'static) -> Errors {
    vec![Box::new(e)]
}

// ---------------------------------------------------------------------------
// Business logic
// ---------------------------------------------------------------------------

fn load_dico(path: &PathBuf) -> Result<Dico, Errors> {
    let content = std::fs::read_to_string(path).map_err(one_err)?;

    let path_str = path.to_string_lossy();
    config::dicofile::parse_dico(&content, &path_str)
}

#[derive(Debug)]
enum SectionEntry<'a> {
    SubSection {
        name: String,
        content: HashMap<String, SectionEntry<'a>>,
    },
    ConfigValue {
        name: String,
        value: &'a ConfigValue,
        keyword: &'a DicoKeyword,
    },
}

impl<'a> SectionEntry<'a> {
    fn content_mut(&mut self) -> &mut HashMap<String, SectionEntry<'a>> {
        match self {
            SectionEntry::SubSection { content, .. } => content,
            SectionEntry::ConfigValue { name, .. } => {
                panic!("expected SubSection, got ConfigValue leaf '{name}'")
            }
        }
    }
}

/// Get the config organized as a tree, with section and subsections
fn build_config_tree<'a>(
    config: &'a HashMap<String, ConfigValue>,
    dico: &'a Dico,
) -> SectionEntry<'a> {
    // Rebuild the hierarchy of the dictionary
    let mut tree = SectionEntry::SubSection {
        name: String::from(""),
        content: HashMap::new(),
    };

    for (kw_name, value) in config {
        let keyword = dico.get(kw_name).expect("ConfigValue for unknown keyword");

        // Walk (and create) intermediate SubSection nodes
        let mut current = &mut tree;
        for section_name in keyword.default_text_desc().classification.iter() {
            // Skip empty name
            if section_name.is_empty() {
                continue;
            }

            current = current
                .content_mut()
                .entry(section_name.clone())
                .or_insert_with(|| SectionEntry::SubSection {
                    name: section_name.clone(),
                    content: HashMap::new(),
                });
        }

        // Insert the leaf at the final level
        current.content_mut().insert(
            kw_name.clone(),
            SectionEntry::ConfigValue {
                name: kw_name.clone(),
                value,
                keyword,
            },
        );
    }

    tree
}

fn as_bullet_list<T: fmt::Display>(lst: &[T]) -> String {
    lst.iter().map(|entry| format!("- {}\n", entry)).collect()
}

fn display_extra_doc(
    render: &mut dyn ConfigViewer,
    doc_requests: &Vec<DocInfo>,
    keyword: &DicoKeyword,
) -> Result<(), Errors> {
    let text_desc = keyword.default_text_desc();
    for doc_req in doc_requests {
        let text = match doc_req {
            DocInfo::Help => text_desc.help.clone(),
            DocInfo::ChoiceOptions => {
                format!(
                    "Possible values\n{}",
                    as_bullet_list(&text_desc.choices_help)
                )
            }
            DocInfo::DefaultValue => match &text_desc.default_val {
                Some(value) => format!("Default value: {}", value),
                None => continue,
            },
            //DocInfo::AlternateKeywordName => {},
            DocInfo::Type => {
                format!("Type: {:?}", keyword.type_)
            }
            DocInfo::Nargs => {
                format!("Size: {}", keyword.nargs)
            }
            DocInfo::Boundaries => match keyword.boundaries {
                Some((min, max)) => format!("Boundaries: [{} ; {}]", min, max),
                None => continue,
            },
        };
        render.emit_comment(text.as_str());
    }

    Ok(())
}

fn display_config<'a>(
    tree: SectionEntry<'a>,
    render: &mut dyn ConfigViewer,
    args: &Args,
) -> Result<(), Errors> {
    let current = tree;
    match current {
        SectionEntry::ConfigValue {
            name,
            value,
            keyword,
        } => {
            let json_value: serde_json::Value = value.try_into().map_err(one_err)?;
            display_extra_doc(render, &args.extra_doc, keyword)?;
            render.emit_kv(name.as_str(), &json_value);
        }
        SectionEntry::SubSection { name, content } => {
            if !name.is_empty() {
                render.emit_section_start(name.as_str());
            }
            for entry in content.into_values() {
                display_config(entry, render, args)?;
            }
            if !name.is_empty() {
                render.emit_section_end();
            }
        }
    }
    Ok(())
}

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

fn dump_config(
    config: &HashMap<String, ConfigValue>,
    dico: &Dico,
    args: &Args,
) -> Result<(), Errors> {
    let tree = build_config_tree(config, dico);

    let stdout = io::stdout();

    let mut renderer = create_config_viewer(stdout.lock(), get_config_viewer_options(args));

    display_config(tree, renderer.as_mut(), args)?;
    renderer.finish();

    Ok(())
}

fn run(args: &Args) -> Result<(), Errors> {
    let dico = load_dico(&args.dico)?;
    let parser = config::casfile::Parser::new(&dico);

    let config = if args.full_dump {
        parser.config_from_file(&args.config)
    } else {
        parser.parse_from_file(&args.config)
    }?;

    if args.dump || args.full_dump {
        dump_config(&config, &dico, args)
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> ExitCode {
    let args = Args::parse();

    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(errors) => {
            for e in errors {
                eprintln!("Error: {e}");
            }
            ExitCode::FAILURE
        }
    }
}
