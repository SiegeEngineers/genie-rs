//! Prints out all AoC hotkey groups and hotkeys, using a language file.

// Example hotkey file path:
// D:\SteamLibrary\steamapps\common\Age2HD\profiles

// Example language file path:
// D:\SteamLibrary\steamapps\common\Age2HD\resources\en\strings\key-value\key-value-strings-utf8.txt

use genie::hki::{self, HotkeyInfo};
use genie::lang::LangFileType;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

/// Displays hotkeys using a language file.
#[derive(Debug, StructOpt)]
#[structopt(name = "Display Language File")]
struct DisplayLang {
    /// The name of the language file.
    #[structopt(name = "lang-file-name")]
    lang_file_name: PathBuf,

    /// The type of the language file.
    ///
    /// One of "dll", "ini", or "key-value".
    #[structopt(name = "file-type")]
    file_type: LangFileType,

    /// The name of the hotkey file.
    #[structopt(name = "hki-file-name")]
    hki_file_name: PathBuf,
}

/// Displays the hotkeys from a hotkey file and language file given to `stdout`.
fn main() -> anyhow::Result<()> {
    let cli_input = DisplayLang::from_args();

    let mut f_lang = File::open(&cli_input.lang_file_name)?;
    let lang_file = cli_input.file_type.read_from(&mut f_lang)?;

    let mut f_hki = File::open(&cli_input.hki_file_name)?;
    let info = HotkeyInfo::from(&mut f_hki)?;
    let aoc_him = hki::default_him();

    println!("{}", info.get_string_from_lang(&lang_file, &aoc_him));
    Ok(())
}
