//! Sets a hotkey in a hotkey file.

use genie::hki::HotkeyInfo;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

// Example hotkey file path:
// D:\SteamLibrary\steamapps\common\Age2HD\profiles

// Example language file path:
// D:\SteamLibrary\steamapps\common\Age2HD\resources\en\strings\key-value\key-value-strings-utf8.txt

/// Sets an individual key binding in a hotkey file.
#[derive(Debug, StructOpt)]
#[structopt(name="Set Hotkey")]
struct SetHotkey {
    /// The name of the hotkey file.
    #[structopt(name="file-name")]
    file_name: PathBuf,

    /// The group index of the hotkey to set.
    #[structopt(name="group-index")]
    group_index: u32,

    /// The index of the hotkey within the group.
    #[structopt(name="hotkey-index")]
    hotkey_index: u32,

    /// The new value of the key binding.
    #[structopt(name="keycode")]
    keycode: i32,

    /// Whether control is held while pressing the hotkey.
    #[structopt(long="ctrl", short="c")]
    ctrl: bool,

    /// Whether alt is held while pressing the hotkey.
    #[structopt(long="alt", short="a")]
    alt: bool,

    /// Whether shift is held while pressing the hotkey.
    #[structopt(long="shift", short="s")]
    shift: bool,
}

/// Executes the CLI.
fn main() -> Result<(), Box<dyn Error>> {
    let cli_input = SetHotkey::from_args();
    let mut f = File::open(&cli_input.file_name)?;
    let info = HotkeyInfo::from(&mut f)?;
    let info = info.bind_key_index(
        cli_input.group_index as usize,
        cli_input.hotkey_index as usize,
        cli_input.keycode, cli_input.ctrl, cli_input.alt, cli_input.shift)?;
    let mut f = File::create(&cli_input.file_name)?;
    info.write_to(&mut f)?;

    let mut f = File::open(&cli_input.file_name)?;
    let info = HotkeyInfo::from(&mut f)?;
    Ok(())
}
