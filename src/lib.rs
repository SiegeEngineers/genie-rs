//! Libraries for reading/writing Age of Empires 2 data files.
//!
//! ## Scenario Files
//!
//! > Supported version range: AoE1 betas through to Age of Empires 2: HD Edition
//!
//! genie-scx can read and write scenario files for almost all Age of Empires versions. When
//! reading a file, the version is detected automatically. When writing a file, you can choose the
//! version to save it as. For example, you can read an HD Edition scenario file, but save it for
//! AoC 1.0c. Note that scenarios that are converted like this may crash the game, because they may
//! refer to terrains or units that do not exist in the different version.
//!
//! ```rust
//! use genie::Scenario;
//! use genie::scx::VersionBundle;
//!
//! /// Read an AoE1 scenario file
//! let mut input = std::fs::File::open("./crates/genie-scx/test/scenarios/Dawn of a New Age.scn")
//!     .expect("failed to open file");
//! let mut output = std::fs::File::create("converted.scx")
//!     .expect("failed to open file");
//!
//! let scen = Scenario::from(&mut input)
//!     .expect("failed to read scenario");
//! scen.write_to_version(&mut output, &VersionBundle::aoc())
//!     .expect("failed to write scenario");
//!
//! std::fs::remove_file("converted.scx")
//!     .expect("failed to delete file");
//! ```
//!
//! ### Implementation Status
//!
//! There aren't many ways to edit a scenario file yet. Initially, we'll work towards the necessary
//! features for proper conversion between AoE versions, especially HD → WololoKingdoms. When that
//! is fairly robust, we'll work on adding methods to edit scenarios and create them from scratch.
//!
//! ## Campaign Files
//!
//! > Supported version range: all versions
//!
//! Campaign files are archives that contain a bunch of scenario files. genie-cpx can extract
//! scenarios from campaign archives and create new campaign archives.
//!
//! ## Hotkey Files
//!
//! > Supported version range: all versions
//!
//! Hotkey files contain groups of key mappings for different game areas.
//!
//! ## Palette Files
//!
//! > Supported version range: all versions
//!
//! Support for palette files is implemented by [chariot_palette](https://github.com/ChariotEngine/Palette/).
//! chariot_palette only supports _reading_ palette files at the moment.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused)]

pub use chariot_palette as pal;
pub use genie_cpx as cpx;
pub use genie_drs as drs;
pub use genie_hki as hki;
pub use genie_lang as lang;
pub use genie_scx as scx;

pub use chariot_palette::read_from as read_palette;
pub use chariot_palette::Palette;
pub use genie_cpx::Campaign;
pub use genie_drs::{DRSReader, DRSWriter};
pub use genie_hki::HotkeyInfo;
pub use genie_lang::LangFile;
pub use genie_scx::Scenario;
