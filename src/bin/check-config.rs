use clap::Parser;
use free_surface::config::configvalue::ConfigValue;
use free_surface::config::{self, dicofile::Dico};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Config file (steering file a.k.a "cas" file) to check
    config: String,

    /// Path of the dico file used to parse the config
    #[arg(long, default_value = "data/dico/telemac2d.dico")]
    dico: PathBuf,

    /// Print the explicitly-set config values after a successful parse
    #[arg(long)]
    dump: bool,

    /// Print all config values (including defaults) after a successful parse
    #[arg(long)]
    full_dump: bool,
}

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

fn dump_config(config: &HashMap<String, ConfigValue>) {
    println!("{:#?}", config);
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
        dump_config(&config);
    }

    Ok(())
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
