use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

use free_surface::config;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Dump the config in case of success
    #[arg(long, default_value = "false")]
    dump: bool,

    /// Path of the dico file used to parse the config
    #[arg(long, default_value = "data/dico/telemac2d.dico")]
    dico: PathBuf,

    /// Config file (steering file a.k.a "cas" file) to check
    config: String,
}

fn get_dico(dico_path: &PathBuf) -> Option<config::dicofile::Dico> {
    let filecontent = std::fs::read_to_string(dico_path);

    match filecontent {
        Ok(content) => {
            let dico_path_str = dico_path.to_str().unwrap();
            let parse_result = config::dicofile::parse_dico(content.as_str(), dico_path_str);
            match parse_result {
                Ok(dico) => Some(dico),
                Err(errors) => {
                    for e in errors {
                        eprintln!("Dico parse error {}", e)
                    }
                    None
                }
            }
        }
        Err(error) => {
            eprintln!("Can't open dico {}", error);
            None
        }
    }
}
fn main() -> ExitCode {
    let args = Args::parse();

    let mut exit_code: ExitCode = ExitCode::SUCCESS;

    if let Some(dico) = get_dico(&args.dico) {
        let parser = config::casfile::Parser::new(&dico);

        match parser.parse_from_file(&args.config) {
            Ok(config) => {
                if args.dump {
                    println!("{:#?}", config)
                }
            }
            Err(errors) => {
                for e in errors {
                    eprintln!("Error: {}", e);
                }
                exit_code = ExitCode::FAILURE;
            }
        }
    }

    exit_code
}
