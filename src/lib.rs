//! Libraries for reading/writing Age of Empires 2 data files.
//!
//! ## Data Files
//!
//! > Supported version range: Age of Empires 2: Age of Kings, Age of Conquerors, HD Edition
//!
//! genie-dat can read data files (empires.dat) for Age of Empires 2. When reading a file, the
//! version is detected automatically, based on the amount of terrains included in the file (since
//! that is hardcoded in each game executable).
//!
//! Writing data files is not yet supported, and many of the things that the library reads are not
//! yet exposed in the public API.
//!
//! ```rust
//! use genie::DatFile;
//! let mut input = std::fs::File::open("./crates/genie-dat/fixtures/aok.dat")
//!     .expect("failed to open file");
//!
//! let dat = DatFile::from(&mut input).expect("failed to parse file");
//! assert_eq!(dat.civilizations.len(), 14);
//! assert_eq!(dat.civilizations[1].name(), "British");
//! ```
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
//! Palette files contain the 256-bit colour palettes used in different areas of the game. Each
//! palette contains up to 256 r, g, b colour values. Both reading and writing is supported.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused)]

pub use genie_cpx as cpx;
// pub use genie_dat as dat;
pub use genie_drs as drs;
pub use genie_hki as hki;
pub use genie_lang as lang;
pub use genie_scx as scx;
pub use jascpal as pal;

pub use genie_cpx::Campaign;
// pub use genie_dat::DatFile;
pub use genie_drs::{DRSReader, DRSWriter};
pub use genie_hki::HotkeyInfo;
pub use genie_lang::LangFile;
pub use genie_scx::Scenario;
pub use jascpal::Palette;
