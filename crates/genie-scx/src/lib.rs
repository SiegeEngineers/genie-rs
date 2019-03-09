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

/// A Scenario file.
pub struct Scenario {
    format: SCXFormat,
    version: VersionBundle,
}

impl Scenario {
    /// Read a scenario file.
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let format = SCXFormat::load_scenario(input)?;
        let version = format.version();

        Ok(Self { format, version })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        self.format.write_to(output, self.version())
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
