//! A reader, writer, and converter for all versions of Age of Empires scenarios.
//!
//! This crate aims to support every single scenario that exists. If a scenario file from any Age
//! of Empires 1 or Age of Empires 2 version does not work, please upload it and file an issue!
mod format;
mod triggers;
mod util;
mod types;

use std::io::{Result, Read, Write};
use format::{
    SCXFormat,
};

pub use types::*;
pub use format::{
    DLCOptions,
    Tile,
    SCXHeader,
};
pub use triggers::{
    Trigger,
    TriggerCondition,
    TriggerEffect,
};

/// All the versions an SCX file uses in a single struct.
#[derive(Debug)]
pub struct VersionBundle {
    /// The version of the 'container' file format.
    pub format: SCXVersion,
    /// The version of the header.
    pub header: u32,
    /// The version of the HD Edition DLC Options, only if `header` >= 3.
    pub dlc_options: i32,
    /// The compressed data version.
    pub data: f32,
    /// The version of embedded bitmaps.
    pub picture: u32,
    /// The version of the victory conditions data.
    pub victory: f32,
    /// The version of the trigger system.
    pub triggers: f64,
}

impl VersionBundle {
    /// A version bundle with the parameters AoK uses by default
    pub fn aok() -> Self {
        unimplemented!()
    }

    /// A version bundle with the parameters AoC uses by default
    pub fn aoc() -> Self {
        Self {
            format: *b"1.21",
            header: 2,
            dlc_options: -1,
            data: 1.22,
            picture: 1,
            victory: 2.0,
            triggers: 1.6,
        }
    }

    /// A version bundle with the parameters UserPatch 1.4 uses by default.
    pub fn userpatch_14() -> Self {
        Self::aoc()
    }

    /// A version bundle with the parameters UserPatch 1.5 uses by default.
    pub fn userpatch_15() -> Self {
        Self::userpatch_14()
    }

    /// A version bundle with the parameters HD Edition uses by default.
    pub fn hd_edition() -> Self {
        Self {
            format: *b"1.21",
            header: 3,
            dlc_options: 1000,
            data: 1.26,
            picture: 3,
            victory: 2.0,
            triggers: 1.6,
        }
    }

    /// Extract version bundle information from a parsed SCX file.
    pub fn from(format: &SCXFormat) -> Self {
        Self {
            format: format.version,
            header: format.header.version,
            data: format.tribe_scen.version(),
            ..Self::aoc()
        }
    }
}

/// A Scenario file.
pub struct Scenario {
    format: SCXFormat,
    version: VersionBundle,
}

impl Scenario {
    /// Read a scenario file.
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let format = SCXFormat::load_scenario(input)?;
        let version = VersionBundle::from(&format);

        Ok(Self { format, version })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        self.format.write_to(output)
    }

    /// Get the format version of this SCX file.
    pub fn format_version(&self) -> SCXVersion {
        self.version().format
    }

    /// Get the header version for this SCX file.
    pub fn header_version(&self) -> u32 {
        self.version().header
    }

    /// Get the data version for this SCX file.
    pub fn data_version(&self) -> f32 {
        self.version().data
    }

    /// Get the header.
    pub fn header(&self) -> &SCXHeader {
        &self.format.header
    }

    /// Get the scenario description.
    pub fn description(&self) -> Option<&str> {
        self.format.tribe_scen.description()
    }

    /// Get the scenario filename.
    pub fn filename(&self) -> &str {
        &self.format.tribe_scen.base.name
    }

    pub fn version(&self) -> &VersionBundle {
        &self.version
    }

    pub fn requires_dlc(&self, dlc: DLCPackage) -> bool {
        match &self.header().dlc_options {
            Some(options) => options.dependencies.iter().any(|dep| *dep == dlc),
            None => false,
        }
    }
}
