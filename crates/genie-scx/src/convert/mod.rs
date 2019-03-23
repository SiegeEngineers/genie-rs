mod aoc_to_wk;
mod hd_to_wk;

use crate::Scenario;

pub use aoc_to_wk::AoCToWK;
pub use hd_to_wk::HDToWK;

#[derive(Debug)]
pub enum ConvertError {
    InvalidVersion,
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConvertError::InvalidVersion => write!(f, "invalid version"),
        }
    }
}

impl std::error::Error for ConvertError {}

pub struct AutoToWK {
}

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
