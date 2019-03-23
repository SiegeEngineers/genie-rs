//! Campaign files store multiple scenario files in one easily distributable chunk.
//!
//! genie-cpx can read and write campaign files using the Campaign and CampaignWriter structs,
//! respectively.
use std::io::{Read, Seek, Write, Error, ErrorKind, Result};

mod read;
mod write;

/// Version identifier for the campaign file format.
///
/// The only existing version is `b"1.00"`.
pub type CPXVersion = [u8; 4];

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
    pub(crate) offset: usize,
    /// Name of the scenario.
    pub name: String,
    /// File name of the scenario.
    pub filename: String,
}

pub use read::Campaign;
pub use write::CampaignWriter;

impl<R> Campaign<R>
    where R: Read + Seek
{
    /// Write the scenario file to an output stream.
    pub fn write_to<W: Write>(&mut self, output: &mut W) -> Result<()> {
        let mut writer = CampaignWriter::new(self.name(), output);

        for i in 0..self.len() {
            let bytes = self.by_index_raw(i)?;
            match (self.get_name(i), self.get_filename(i)) {
                (Some(name), Some(filename)) => {
                    writer.add_raw(name, filename, bytes);
                },
                _ => return Err(Error::new(ErrorKind::Other, "missing data for scenario"))
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
    fn rebuild_cpx() {
        let instream = File::open("./test/campaigns/Armies at War A Combat Showcase.cpn").unwrap();
        let mut outstream = vec![];
        let mut incpx = Campaign::from(instream).unwrap();
        incpx.write_to(&mut outstream).unwrap();

        let mut written_cpx = Campaign::from(Cursor::new(outstream)).unwrap();
        assert_eq!(written_cpx.name(), incpx.name());
        assert_eq!(written_cpx.len(), incpx.len());
        assert_eq!(written_cpx.by_index_raw(0).unwrap(), incpx.by_index_raw(0).unwrap());
    }
}
