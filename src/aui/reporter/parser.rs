// # Abstract parser reporting interface
//
// The abstract parser reporting interface is a user interface element allowing to report
// info about parsing a text file.
// This includes errors, warnings and hints.

use std::io::Write;

use crate::config::textloc::{LocalizedError, UNKNOWN_FILE};

//type ErrorReportFct=Fn(msg: dyn LocalizedError);

fn loc_prefix<W: Write>(msg: &dyn LocalizedError, w: &mut W) -> std::io::Result<()> {
    let native_filename = msg.filename().as_os_str();

    let filename = if native_filename.is_empty() {
        String::from(UNKNOWN_FILE)
    } else {
        format!("{}", native_filename.display())
    };
    if msg.column() == 0 {
        write!(w, "{}:{} ", filename, msg.line())
    } else {
        write!(w, "{}:{}:{} ", filename, msg.line(), msg.column())
    }
}

/// Report a parsing error
pub fn parse_error(msg: &dyn LocalizedError) {
    loc_prefix(msg, &mut std::io::stderr()).unwrap();
    eprintln!("Error: {}", msg)
}

/// Report a parsing warning
pub fn parse_warn(msg: &dyn LocalizedError) {
    loc_prefix(msg, &mut std::io::stderr()).unwrap();
    eprintln!("Warning: {:?}", msg)
}

/// Report a parsing hint
pub fn parse_hint(msg: &dyn LocalizedError) {
    loc_prefix(msg, &mut std::io::stderr()).unwrap();
    eprintln!("Hint: {:?}", msg)
}
