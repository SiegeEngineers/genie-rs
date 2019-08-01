//! Displays civilizations from a specified dat file.

use genie::DatFile;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

/// Displays an individual hotkey from a hotkey file.
#[derive(Debug, StructOpt)]
#[structopt(name = "displaycivs")]
struct DisplayCivs {
    /// The name of the dat file.
    file_name: PathBuf,
}

/// Executes the CLI.
fn main() -> Result<(), Box<dyn Error>> {
    let cli_input = DisplayCivs::from_args();
    let mut f = File::open(&cli_input.file_name)?;
    let dat = DatFile::from(&mut f)?;
    for civ in &dat.civilizations {
        println!("{}", civ.name());
    }
    Ok(())
}
