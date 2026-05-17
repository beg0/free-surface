use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

use free_surface::storage::selafin;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Selfain file to view
    sfl_file: PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let mut exit_code: ExitCode = ExitCode::SUCCESS;

    match selafin::parse_file(args.sfl_file) {
        Ok(slf) => println!("{:#?}", slf),
        Err(error) => {
            eprintln!("Error: {}", error);
            exit_code = ExitCode::FAILURE;
        }
    };

    exit_code
}
