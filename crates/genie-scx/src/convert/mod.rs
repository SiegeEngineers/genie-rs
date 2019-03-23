mod aoc_to_wk;
mod hd_to_wk;

use crate::Scenario;

pub use aoc_to_wk::AoCToWK;
pub use hd_to_wk::HDToWK;

pub struct AutoToWK {
}

impl Default for AutoToWK {
    fn default() -> Self {
        AutoToWK {}
    }
}

impl AutoToWK {
    pub fn convert(&self, scen: &mut Scenario) -> Result<(), &'static str> {
        if scen.version().is_hd_edition() {
            HDToWK::default().convert(scen);
            Ok(())
        } else if scen.version().is_aok() || scen.version().is_aoc() {
            // TODO check if this is already a WK scenario … somehow
            AoCToWK::default().convert(scen);
            Ok(())
        } else {
            Err("Invalid version")
        }
    }
}
