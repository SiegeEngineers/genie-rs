//! Campaign files store multiple scenario files in one easily distributable chunk.
//!
//! genie-cpx can read and write campaign files using the Campaign and CampaignWriter structs,
//! respectively.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused)]

use std::io::{Read, Seek, Write};

mod read;
mod write;

pub use read::{Campaign, ReadCampaignError};
pub use write::{CampaignWriter, WriteCampaignError};

/// Version identifier for the campaign file format.
///
/// The only existing version is `b"1.00"`.
pub type CPXVersion = [u8; 4];

/// Version identifier for AoE1, AoE2, and AoE2: HD campaign files.
pub const AOE_AOK: CPXVersion = *b"1.00";
/// Version identifier for AoE1: Definitive Edition campaign files.
pub const AOE1_DE: CPXVersion = *b"1.10";

/// Campaign header.
#[derive(Debug, Clone)]
pub(crate) struct CampaignHeader {
    /// File format version.
    pub(crate) version: CPXVersion,
    /// Name of the campaign.
    pub(crate) name: String,
    /// Amount of scenario files in this campaign.
    pub(crate) num_scenarios: usize,
}

impl CampaignHeader {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            version: *b"1.00",
            name: name.to_string(),
            num_scenarios: 0,
        }
    }
}

/// Data about a scenario in the campaign file.
#[derive(Debug, Clone)]
pub struct ScenarioMeta {
    /// Size in bytes of the scenario file.
    pub size: usize,
    /// Offset in bytes of the scenario file within the campaign file.
    pub(crate) offset: usize,
    /// Name of the scenario.
    pub name: String,
    /// File name of the scenario.
    pub filename: String,
}

impl<R> Campaign<R>
where
    R: Read + Seek,
{
    /// Write the scenario file to an output stream, using the same version as when reading it.
    pub fn write_to<W: Write>(&mut self, output: &mut W) -> Result<(), WriteCampaignError> {
        self.write_to_version(output, self.version())
    }

    /// Write the scenario file to an output stream with the given version.
    pub fn write_to_version<W: Write>(
        &mut self,
        output: &mut W,
        version: CPXVersion,
    ) -> Result<(), WriteCampaignError> {
        let mut writer = CampaignWriter::new(self.name(), output).version(version);

        for i in 0..self.len() {
            let bytes = self
                .by_index_raw(i)
                .map_err(|_| WriteCampaignError::NotFoundError(i))?;
            match (self.get_name(i), self.get_filename(i)) {
                (Some(name), Some(filename)) => {
                    writer.add_raw(name, filename, bytes);
                }
                _ => return Err(WriteCampaignError::NotFoundError(i)),
            }
        }

        let _output = writer.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Cursor;

    #[test]
    fn rebuild_cpx() -> Result<(), Box<dyn std::error::Error>> {
        let instream = File::open("./test/campaigns/Armies at War A Combat Showcase.cpn")?;
        let mut outstream = vec![];
        let mut incpx = Campaign::from(instream)?;
        incpx.write_to(&mut outstream)?;

        let mut written_cpx = Campaign::from(Cursor::new(outstream))?;
        assert_eq!(written_cpx.name(), incpx.name());
        assert_eq!(written_cpx.len(), incpx.len());
        assert_eq!(written_cpx.by_index_raw(0)?, incpx.by_index_raw(0)?);
        Ok(())
    }

    #[test]
    fn rebuild_cpn_de() -> Result<(), Box<dyn std::error::Error>> {
        let instream = File::open("./test/campaigns/10 The First Punic War.aoecpn")?;
        let mut outstream = vec![];
        let mut incpx = Campaign::from(instream)?;
        incpx.write_to(&mut outstream)?;

        let mut written_cpx = Campaign::from(Cursor::new(outstream))?;
        assert_eq!(written_cpx.name(), incpx.name());
        assert_eq!(written_cpx.len(), incpx.len());
        assert_eq!(written_cpx.version(), incpx.version());
        assert_eq!(written_cpx.get_name(0), incpx.get_name(0));
        assert_eq!(written_cpx.get_filename(0), incpx.get_filename(0));
        assert_eq!(written_cpx.by_index_raw(0)?, incpx.by_index_raw(0)?);
        Ok(())
    }
}
