//! Displays a hotkey to `stdout`.

use genie::hki::HotkeyInfo;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

// Example hotkey file path:
// D:\SteamLibrary\steamapps\common\Age2HD\profiles

// Example language file path:
// D:\SteamLibrary\steamapps\common\Age2HD\resources\en\strings\key-value\key-value-strings-utf8.txt

/// Displays an individual hotkey from a hotkey file.
#[derive(Debug, StructOpt)]
#[structopt(name = "Set Hotkey")]
struct DisplayHotkey {
    /// The name of the hotkey file.
    #[structopt(name = "file-name")]
    file_name: PathBuf,

    /// The group index of the hotkey to display.
    #[structopt(name = "group-index")]
    group_index: usize,

    /// The index of the hotkey within the group.
    #[structopt(name = "hotkey-index")]
    hotkey_index: usize,
}

/// Executes the CLI.
fn main() -> Result<(), Box<dyn Error>> {
    let cli_input = DisplayHotkey::from_args();
    let mut f = File::open(&cli_input.file_name)?;
    let info = HotkeyInfo::from(&mut f)?;
    let group = info.group(cli_input.group_index).unwrap();
    let hotkey = group.hotkey(cli_input.hotkey_index).unwrap();
    println!("{}", hotkey);
    Ok(())
}
