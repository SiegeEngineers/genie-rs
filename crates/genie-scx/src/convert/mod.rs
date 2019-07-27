//! Automated scenario conversions.
//!
//! This module implements conversions between different scenario formats and game versions.
mod aoc_to_wk;
mod hd_to_wk;

use crate::Scenario;

pub use aoc_to_wk::AoCToWK;
pub use hd_to_wk::HDToWK;

/// Error indicating scenario conversion failure.
#[derive(Debug)]
pub enum ConvertError {
    /// The input scenario version is not supported by the converter.
    InvalidVersion,
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertError::InvalidVersion => write!(f, "invalid version"),
        }
    }
}

impl std::error::Error for ConvertError {}

/// Convert an AoC or HD Edition scenario file to a WololoKingdoms one.
///
/// It will auto-detect the version of the file, and output a WK compatible scenario.
/// AoC scenarios will have their unit and terrain IDs switched around so they have the correct
/// look in WK.
/// HD Edition scenarios will have all the new unit and terrain IDs mapped to WK IDs.
///
/// ## Usage
///
/// ```rust,ignore
/// use genie_scx::convert::AutoToWK;
/// AutoToWK::default().convert(&mut scenario)?
/// ```
pub struct AutoToWK {}

impl Default for AutoToWK {
    fn default() -> Self {
        AutoToWK {}
    }
}

impl AutoToWK {
    pub fn convert(&self, scen: &mut Scenario) -> Result<(), ConvertError> {
        if scen.version().is_hd_edition() {
            HDToWK::default().convert(scen)
        } else if scen.version().is_aok() || scen.version().is_aoc() {
            // TODO check if this is already a WK scenario … somehow
            AoCToWK::default().convert(scen)
        } else {
            Err(ConvertError::InvalidVersion)
        }
    }
}
