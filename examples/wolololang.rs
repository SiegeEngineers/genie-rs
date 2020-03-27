//! Converts a key-value language file to an ini language file, removing all
//! strings with a string key name instead of a numeric key name.
//!
//! The order in which string keys are written to the output file is currently
//! unspecified.

use genie::lang::LangFileType::KeyValue;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

/// Struct to collect input and output file paths from the command line.
#[derive(Debug, StructOpt)]
#[structopt(name = "Wololo Language File")]
struct WololoLang {
    /// The path of the input language key-value file.
    #[structopt(name = "path-in")]
    path_in: PathBuf,

    /// The path of the output language ini file.
    ///
    /// Overwrites this file if it already exists.
    #[structopt(name = "path-out")]
    path_out: PathBuf,
}

/// Collects command line input and converts the specified key-value language
/// file into an ini language file.
fn main() -> anyhow::Result<()> {
    let cli_input = WololoLang::from_args();
    let mut f_in = File::open(&cli_input.path_in)?;
    let lang_file = KeyValue.read_from(&mut f_in)?;
    let mut f_out = File::create(&cli_input.path_out)?;
    lang_file.write_to_ini(&mut f_out)?;
    Ok(())
}
