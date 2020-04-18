//! A reader, writer, and converter for all versions of Age of Empires scenarios.
//!
//! This crate aims to support every single scenario that exists. If a scenario file from any Age
//! of Empires 1 or Age of Empires 2 version does not work, please upload it and file an issue!

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused)]

mod ai;
mod bitmap;
pub mod convert;
mod format;
mod header;
mod map;
mod player;
mod triggers;
mod types;
mod victory;

use format::SCXFormat;
use genie_support::{ReadStringError, WriteStringError};
use std::io::{self, Read, Write};

pub use ai::ParseAIErrorCodeError;
pub use format::{ScenarioObject, TribeScen};
pub use genie_support::{DecodeStringError, EncodeStringError};
pub use genie_support::{StringKey, UnitTypeID};
pub use header::{DLCOptions, SCXHeader};
pub use map::{Map, Tile};
pub use triggers::{Trigger, TriggerCondition, TriggerEffect, TriggerSystem};
pub use types::*;
pub use victory::{VictoryConditions, VictoryEntry, VictoryPointEntry};

/// Error type for SCX methods, containing all types of errors that may occur while reading or
/// writing scenario files.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The scenario that's attempted to be read does not contain a file name.
    #[error("must have a file name")]
    MissingFileNameError,
    /// Attempted to read a scenario with an unsupported format version identifier.
    #[error("unsupported format version {:?}", .0)]
    UnsupportedFormatVersionError(SCXVersion),
    /// Attempted to write a scenario with disabled technologies, to a version that doesn't support
    /// this many disabled technologies.
    #[error("too many disabled techs: got {}, but requested version supports up to 20", .0)]
    TooManyDisabledTechsError(i32),
    /// Attempted to write a scenario with disabled technologies, to a version that doesn't support
    /// disabling technologies.
    #[error("requested version does not support disabling techs")]
    CannotDisableTechsError,
    /// Attempted to write a scenario with disabled units, to a version that doesn't support
    /// disabling units.
    #[error("requested version does not support disabling units")]
    CannotDisableUnitsError,
    /// Attempted to write a scenario with disabled buildings, to a version that doesn't support
    /// this many disabled buildings.
    #[error("too many disabled buildings: got {}, but requested version supports up to {}", .0, .1)]
    TooManyDisabledBuildingsError(i32, i32),
    /// Attempted to write a scenario with disabled buildings, to a version that doesn't support
    /// disabling buildings.
    #[error("requested version does not support disabling buildings")]
    CannotDisableBuildingsError,
    /// Failed to decode a string from the scenario file, probably because of a wrong encoding.
    #[error(transparent)]
    DecodeStringError(#[from] DecodeStringError),
    /// Failed to encode a string into the scenario file, probably because of a wrong encoding.
    #[error(transparent)]
    EncodeStringError(#[from] EncodeStringError),
    /// The given ID is not a known diplomatic stance.
    #[error(transparent)]
    ParseDiplomaticStanceError(#[from] ParseDiplomaticStanceError),
    /// The given ID is not a known data set.
    #[error(transparent)]
    ParseDataSetError(#[from] ParseDataSetError),
    /// The given ID is not a known HD Edition DLC.
    #[error(transparent)]
    ParseDLCPackageError(#[from] ParseDLCPackageError),
    /// The given ID is not a known starting age in AoE1 or AoE2.
    #[error(transparent)]
    ParseStartingAgeError(#[from] ParseStartingAgeError),
    /// The given ID is not a known error code.
    #[error(transparent)]
    ParseAIErrorCodeError(#[from] ParseAIErrorCodeError),
    /// An error occurred while reading or writing.
    #[error(transparent)]
    IoError(#[from] io::Error),
}

impl From<ReadStringError> for Error {
    fn from(err: ReadStringError) -> Error {
        match err {
            ReadStringError::IoError(err) => Error::IoError(err),
            ReadStringError::DecodeStringError(err) => Error::DecodeStringError(err),
        }
    }
}

impl From<WriteStringError> for Error {
    fn from(err: WriteStringError) -> Error {
        match err {
            WriteStringError::IoError(err) => Error::IoError(err),
            WriteStringError::EncodeStringError(err) => Error::EncodeStringError(err),
        }
    }
}

/// Result type for SCX methods.
pub type Result<T> = std::result::Result<T, Error>;

/// A Scenario file.
#[derive(Debug, Clone)]
pub struct Scenario {
    format: SCXFormat,
    version: VersionBundle,
}

impl Scenario {
    /// Read a scenario file.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let format = SCXFormat::load_scenario(&mut input)?;
        let version = format.version();

        Ok(Self { format, version })
    }

    /// Read a scenario file.
    #[deprecated = "Use Scenario::read_from instead."]
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        Self::read_from(input)
    }

    /// Write the scenario file to an output stream.
    ///
    /// Equivalent to `scen.write_to_version(scen.version())`.
    #[inline]
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        self.format.write_to(output, self.version())
    }

    /// Write the scenario file to an output stream, targeting specific game versions.
    #[inline]
    pub fn write_to_version<W: Write>(
        &self,
        output: &mut W,
        version: &VersionBundle,
    ) -> Result<()> {
        self.format.write_to(output, version)
    }

    /// Get the format version of this SCX file.
    #[inline]
    pub fn format_version(&self) -> SCXVersion {
        self.version().format
    }

    /// Get the header version for this SCX file.
    #[inline]
    pub fn header_version(&self) -> u32 {
        self.version().header
    }

    /// Get the data version for this SCX file.
    #[inline]
    pub fn data_version(&self) -> f32 {
        self.version().data
    }

    /// Get the header.
    #[inline]
    pub fn header(&self) -> &SCXHeader {
        &self.format.header
    }

    /// Get the scenario description.
    #[inline]
    pub fn description(&self) -> Option<&str> {
        self.format.tribe_scen.description()
    }

    /// Get the scenario filename.
    #[inline]
    pub fn filename(&self) -> &str {
        &self.format.tribe_scen.base.name
    }

    /// Get data about the game versions this scenario file was made for.
    #[inline]
    pub fn version(&self) -> &VersionBundle {
        &self.version
    }

    /// Check if this scenario requires the given DLC (for HD Edition scenarios only).
    #[inline]
    pub fn requires_dlc(&self, dlc: DLCPackage) -> bool {
        match &self.header().dlc_options {
            Some(options) => options.dependencies.iter().any(|dep| *dep == dlc),
            None => false,
        }
    }

    /// Get the UserPatch mod name of the mod that was used to create this scenario.
    ///
    /// This returns the short name, like "WK" for WololoKingdoms or "aoc" for Age of Chivalry.
    #[inline]
    pub fn mod_name(&self) -> Option<&str> {
        self.format.mod_name()
    }

    /// Iterate over all the objects placed in the scenario.
    #[inline]
    pub fn objects(&self) -> impl Iterator<Item = &ScenarioObject> {
        self.format
            .player_objects
            .iter()
            .map(|list| list.iter())
            .flatten()
    }

    /// Iterate mutably over all the objects placed in the scenario.
    #[inline]
    pub fn objects_mut(&mut self) -> impl Iterator<Item = &mut ScenarioObject> {
        self.format
            .player_objects
            .iter_mut()
            .map(|list| list.iter_mut())
            .flatten()
    }

    /// Get the map/terrain data for this scenario.
    #[inline]
    pub fn map(&self) -> &Map {
        &self.format.map
    }

    /// Get the (mutable) map/terrain data for this scenario.
    #[inline]
    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.format.map
    }

    /// Get trigger data for this scenario if it exists.
    #[inline]
    pub fn triggers(&self) -> Option<&TriggerSystem> {
        self.format.triggers.as_ref()
    }

    /// Get (mutable) trigger data for this scenario if it exists.
    #[inline]
    pub fn triggers_mut(&mut self) -> Option<&mut TriggerSystem> {
        self.format.triggers.as_mut()
    }
}
