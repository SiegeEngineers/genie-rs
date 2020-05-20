//! Displays the key value pairs in a language file.

use genie::lang::LangFileType;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

/// Displays the strings from a language file.
#[derive(Debug, StructOpt)]
#[structopt(name = "Display Language File")]
struct DisplayLang {
    /// The name of the language file.
    #[structopt(name = "file-name")]
    file_name: PathBuf,

    /// The type of the language file.
    ///
    /// One of "dll", "ini", or "key-value".
    #[structopt(name = "file-type")]
    file_type: LangFileType,
}

/// Prints the key value pairs of an input language file to `stdout`.
fn main() -> anyhow::Result<()> {
    let cli_input = DisplayLang::from_args();
    let mut f = File::open(&cli_input.file_name)?;
    let lang_file = cli_input.file_type.read_from(&mut f)?;
    println!("{}", lang_file);
    Ok(())
}
