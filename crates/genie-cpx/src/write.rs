use crate::{CampaignHeader, ScenarioMeta};
use byteorder::{WriteBytesExt, LE};
use genie_scx::{Result as SCXResult, Scenario};
use std::io::{self, Write};

/// Type for errors that could occur during writing.
#[derive(Debug)]
pub enum WriteCampaignError {
    /// An I/O error occurred during writing.
    IoError(io::Error),
    /// A scenario could not be found, either because the original campaign file was corrupt, or
    /// the scenario file exists but could not be parsed.
    NotFoundError(usize),
}

impl From<io::Error> for WriteCampaignError {
    fn from(err: io::Error) -> WriteCampaignError {
        WriteCampaignError::IoError(err)
    }
}

impl std::fmt::Display for WriteCampaignError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteCampaignError::IoError(err) => write!(f, "{}", err),
            WriteCampaignError::NotFoundError(n) => {
                write!(f, "missing scenario data for index {}", n)
            }
        }
    }
}

impl std::error::Error for WriteCampaignError {}

/// Write the campaign header to the `output` stream.
fn write_campaign_header<W: Write>(header: &CampaignHeader, output: &mut W) -> io::Result<()> {
    assert!(header.num_scenarios < std::i32::MAX as usize);

    output.write_all(&header.version)?;
    let mut name_bytes = header.name.as_bytes().to_vec();
    assert!(name_bytes.len() < 256);
    name_bytes.extend(vec![0; 256 - name_bytes.len()]);
    output.write_all(&name_bytes)?;
    output.write_i32::<LE>(header.num_scenarios as i32)?;
    Ok(())
}

/// Write metadata for a single scenario into the `output` stream.
fn write_scenario_meta<W: Write>(meta: &ScenarioMeta, output: &mut W) -> io::Result<()> {
    assert!(meta.size < std::i32::MAX as usize);
    assert!(meta.offset < std::i32::MAX as usize);

    output.write_i32::<LE>(meta.size as i32)?;
    output.write_i32::<LE>(meta.offset as i32)?;

    let mut name_bytes = meta.name.as_bytes().to_vec();
    assert!(name_bytes.len() < 255);
    name_bytes.extend(vec![0; 255 - name_bytes.len()]);
    output.write_all(&name_bytes)?;

    let mut filename_bytes = meta.filename.as_bytes().to_vec();
    assert!(filename_bytes.len() < 255);
    filename_bytes.extend(vec![0; 255 - filename_bytes.len()]);
    output.write_all(&filename_bytes)?;

    Ok(())
}

/// Describes a scenario file to be added to the campaign.
struct CampaignEntry {
    name: String,
    filename: String,
    bytes: Vec<u8>,
}

impl CampaignEntry {
    /// Get the user-visible name of this entry.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get the file name of this entry.
    fn filename(&self) -> &str {
        &self.filename
    }

    /// Get the size in bytes of this entry.
    fn size(&self) -> usize {
        self.bytes.len()
    }

    /// Get the byte array for this entry.
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// A campaign file writer. Instantiate it, then add scenario files to it.
///
/// This has to keep all scenario files in memory until the file is written, either on a call to `flush()` or implicitly when the struct is dropped.
pub struct CampaignWriter<W: Write> {
    writer: W,
    header: CampaignHeader,
    scenarios: Vec<CampaignEntry>,
}

impl<W: Write> CampaignWriter<W> {
    /// Create a new campaign with user-visible name `name`, writing to the `writer` stream.
    pub fn new(name: &str, writer: W) -> Self {
        Self {
            writer,
            header: CampaignHeader::new(name),
            scenarios: vec![],
        }
    }

    /// Add a scenario (as a byte array) to this campaign.
    pub fn add_raw(&mut self, name: &str, filename: &str, scx: Vec<u8>) {
        self.scenarios.push(CampaignEntry {
            name: name.to_owned(),
            filename: filename.to_owned(),
            bytes: scx,
        });
    }

    /// Add a Scenario instance from genie-scx to this campaign.
    pub fn add(&mut self, name: &str, scx: &Scenario) -> SCXResult<()> {
        let mut bytes = vec![];
        scx.write_to(&mut bytes)?;
        self.scenarios.push(CampaignEntry {
            name: name.to_owned(),
            filename: scx.filename().to_owned(),
            bytes,
        });
        Ok(())
    }

    /// Consume the `CampaignWriter` instance, returning the inner `Write` instance.
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Write the campaign header.
    fn write_header(&mut self) -> io::Result<()> {
        self.header.num_scenarios = self.scenarios.len();
        write_campaign_header(&self.header, &mut self.writer)
    }

    /// Write the scenario metadata block.
    fn write_metas(&mut self) -> io::Result<()> {
        let mut offset = 256 + 8 + self.scenarios.len() * (255 + 255 + 8);
        for scen in &self.scenarios {
            let meta = ScenarioMeta {
                size: scen.size(),
                offset,
                name: scen.name().to_owned(),
                filename: scen.filename().to_owned(),
            };
            write_scenario_meta(&meta, &mut self.writer)?;
            offset += scen.size();
        }
        Ok(())
    }

    /// Write the scenario data.
    fn write_scenarios(&mut self) -> io::Result<()> {
        for scen in &self.scenarios {
            self.writer.write_all(scen.bytes())?;
        }
        Ok(())
    }

    /// Write the scenarios to the output stream, consuming the CampaignWriter object.
    ///
    /// Returns the inner `Write`.
    pub fn flush(mut self) -> io::Result<W> {
        self.write_header()?;
        self.write_metas()?;
        self.write_scenarios()?;

        Ok(self.into_inner())
    }
}
