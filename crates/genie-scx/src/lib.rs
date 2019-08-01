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
mod util;
mod victory;

use format::SCXFormat;
use std::io::{self, Read, Write};

pub use format::ScenarioObject;
pub use genie_support::{StringID, UnitTypeID};
pub use header::{DLCOptions, SCXHeader};
pub use map::{Map, Tile};
pub use triggers::{Trigger, TriggerCondition, TriggerEffect, TriggerSystem};
pub use types::*;
pub use util::{DecodeStringError, EncodeStringError};

/// Error type for SCX methods, containing all types of errors that may occur while reading or
/// writing scenario files.
#[derive(Debug)]
pub enum Error {
    /// The scenario that's attempted to be read does not contain a file name.
    MissingFileNameError,
    /// Attempted to read a scenario with an unsupported format version identifier.
    UnsupportedFormatVersionError(SCXVersion),
    /// Attempted to write a scenario with disabled technologies, to a version that doesn't support
    /// this many disabled technologies.
    TooManyDisabledTechsError(i32),
    /// Attempted to write a scenario with disabled technologies, to a version that doesn't support
    /// disabling technologies.
    CannotDisableTechsError,
    /// Attempted to write a scenario with disabled units, to a version that doesn't support
    /// disabling units.
    CannotDisableUnitsError,
    /// Attempted to write a scenario with disabled buildings, to a version that doesn't support
    /// this many disabled buildings.
    TooManyDisabledBuildingsError(i32, i32),
    /// Attempted to write a scenario with disabled buildings, to a version that doesn't support
    /// disabling buildings.
    CannotDisableBuildingsError,
    /// Failed to decode a string from the scenario file, probably because of a wrong encoding.
    DecodeStringError(DecodeStringError),
    /// Failed to encode a string into the scenario file, probably because of a wrong encoding.
    EncodeStringError(EncodeStringError),
    /// The given ID is not a known diplomatic stance.
    ParseDiplomaticStanceError(ParseDiplomaticStanceError),
    /// The given ID is not a known data set.
    ParseDataSetError(ParseDataSetError),
    /// The given ID is not a known HD Edition DLC.
    ParseDLCPackageError(ParseDLCPackageError),
    /// The given ID is not a known starting age in AoE1 or AoE2.
    ParseStartingAgeError(ParseStartingAgeError),
    /// An error occurred while reading or writing.
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<util::ReadStringError> for Error {
    fn from(err: util::ReadStringError) -> Error {
        match err {
            util::ReadStringError::IoError(err) => Error::IoError(err),
            util::ReadStringError::DecodeStringError(err) => Error::DecodeStringError(err),
        }
    }
}

impl From<util::WriteStringError> for Error {
    fn from(err: util::WriteStringError) -> Error {
        match err {
            util::WriteStringError::IoError(err) => Error::IoError(err),
            util::WriteStringError::EncodeStringError(err) => Error::EncodeStringError(err),
        }
    }
}

macro_rules! error_impl_from {
    ($from:ident) => {
        impl From<$from> for Error {
            fn from(err: $from) -> Error {
                Error::$from(err)
            }
        }
    };
}

error_impl_from!(ParseDiplomaticStanceError);
error_impl_from!(ParseDataSetError);
error_impl_from!(ParseDLCPackageError);
error_impl_from!(ParseStartingAgeError);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingFileNameError => write!(f, "must have a file name"),
            Error::UnsupportedFormatVersionError(version) => {
                write!(f, "unsupported format version {:?}", version)
            }
            Error::TooManyDisabledTechsError(n) => write!(
                f,
                "too many disabled techs: got {}, but requested version supports up to 20",
                n
            ),
            Error::TooManyDisabledBuildingsError(n, max) => write!(
                f,
                "too many disabled buildings: got {}, but requested version supports up to {}",
                n, max
            ),
            Error::CannotDisableTechsError => {
                write!(f, "requested version does not support disabling techs")
            }
            Error::CannotDisableUnitsError => {
                write!(f, "requested version does not support disabling units")
            }
            Error::CannotDisableBuildingsError => {
                write!(f, "requested version does not support disabling buildings")
            }
            Error::IoError(err) => write!(f, "{}", err),
            Error::DecodeStringError(err) => write!(f, "{}", err),
            Error::EncodeStringError(err) => write!(f, "{}", err),
            Error::ParseDiplomaticStanceError(err) => write!(f, "{}", err),
            Error::ParseDataSetError(err) => write!(f, "{}", err),
            Error::ParseDLCPackageError(err) => write!(f, "{}", err),
            Error::ParseStartingAgeError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for SCX methods.
pub type Result<T> = std::result::Result<T, Error>;

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

    /// Write the scenario file to an output stream.
    ///
    /// Equivalent to `scen.write_to_version(scen.version())`.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        self.format.write_to(output, self.version())
    }

    /// Write the scenario file to an output stream, targeting specific game versions.
    pub fn write_to_version<W: Write>(
        &self,
        output: &mut W,
        version: &VersionBundle,
    ) -> Result<()> {
        self.format.write_to(output, version)
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

    /// Get data about the game versions this scenario file was made for.
    pub fn version(&self) -> &VersionBundle {
        &self.version
    }

    /// Check if this scenario requires the given DLC (for HD Edition scenarios only).
    pub fn requires_dlc(&self, dlc: DLCPackage) -> bool {
        match &self.header().dlc_options {
            Some(options) => options.dependencies.iter().any(|dep| *dep == dlc),
            None => false,
        }
    }

    /// Iterate over all the objects placed in the scenario.
    pub fn objects(&self) -> impl Iterator<Item = &ScenarioObject> {
        self.format
            .player_objects
            .iter()
            .map(|list| list.iter())
            .flatten()
    }

    /// Iterate mutably over all the objects placed in the scenario.
    pub fn objects_mut(&mut self) -> impl Iterator<Item = &mut ScenarioObject> {
        self.format
            .player_objects
            .iter_mut()
            .map(|list| list.iter_mut())
            .flatten()
    }

    /// Get the map/terrain data for this scenario.
    pub fn map(&self) -> &Map {
        &self.format.map
    }

    /// Get the (mutable) map/terrain data for this scenario.
    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.format.map
    }

    /// Get trigger data for this scenario if it exists.
    pub fn triggers(&self) -> Option<&TriggerSystem> {
        self.format.triggers.as_ref()
    }

    /// Get (mutable) trigger data for this scenario if it exists.
    pub fn triggers_mut(&mut self) -> Option<&mut TriggerSystem> {
        self.format.triggers.as_mut()
    }
}
