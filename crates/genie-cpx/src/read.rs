use crate::{CPXVersion, CampaignHeader, ScenarioMeta};
use chardet::{detect as detect_encoding};
use encoding_rs::Encoding;
use byteorder::{ReadBytesExt, LE};
use genie_scx::{self as scx, Scenario};
use std::io::{self, Cursor, Read, Seek, SeekFrom};

#[derive(Debug)]
pub enum ReadCampaignError {
    DecodeStringError,
    IoError(io::Error),
    MissingNameError,
    NotFoundError,
    ParseSCXError(scx::Error),
}

impl From<io::Error> for ReadCampaignError {
    fn from(err: io::Error) -> ReadCampaignError {
        ReadCampaignError::IoError(err)
    }
}

impl From<scx::Error> for ReadCampaignError {
    fn from(err: scx::Error) -> ReadCampaignError {
        ReadCampaignError::ParseSCXError(err)
    }
}

impl std::fmt::Display for ReadCampaignError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadCampaignError::DecodeStringError => write!(f, "invalid string"),
            ReadCampaignError::IoError(err) => write!(f, "{}", err),
            ReadCampaignError::MissingNameError => {
                write!(f, "campaign or scenario must have a name")
            }
            ReadCampaignError::NotFoundError => write!(f, "scenario not found in campaign"),
            ReadCampaignError::ParseSCXError(err) => write!(f, "{}", err),
        }
    }
}

type Result<T> = std::result::Result<T, ReadCampaignError>;

/// Decode a string with unknown encoding.
fn decode_str(bytes: &[u8]) -> Result<String> {
    let (encoding_name, confidence, language) = detect_encoding(&bytes);
    Encoding::for_label(encoding_name.as_bytes())
        .ok_or(ReadCampaignError::DecodeStringError)
        .and_then(|encoding| {
            let (decoded, _enc, failed) = encoding.decode(&bytes);
            if failed {
                return Err(ReadCampaignError::DecodeStringError)
            }
            Ok(decoded.to_string())
        })
}

pub fn read_fixed_str<R: Read>(input: &mut R, len: usize) -> Result<Option<String>> {
    let mut bytes = vec![0; len];
    input.read_exact(&mut bytes[0..len])?;

    if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
        bytes.truncate(end);
    }
    if bytes.is_empty() {
        Ok(None)
    } else {
        decode_str(&bytes).map(Some)
    }
}

fn read_campaign_header<R: Read>(input: &mut R) -> Result<CampaignHeader> {
    let mut version = [0; 4];
    input.read_exact(&mut version)?;
    let name = read_fixed_str(input, 256)?.ok_or(ReadCampaignError::MissingNameError)?;
    let num_scenarios = input.read_i32::<LE>()? as usize;

    Ok(CampaignHeader {
        version,
        name,
        num_scenarios,
    })
}

fn read_scenario_meta<R: Read>(input: &mut R) -> Result<ScenarioMeta> {
    let size = input.read_i32::<LE>()? as usize;
    let offset = input.read_i32::<LE>()? as usize;
    let name = read_fixed_str(input, 255)?.ok_or(ReadCampaignError::MissingNameError)?;
    let filename = read_fixed_str(input, 255)?.ok_or(ReadCampaignError::MissingNameError)?;
    let mut padding = [0; 2];
    input.read_exact(&mut padding)?;

    Ok(ScenarioMeta {
        size,
        offset,
        name,
        filename,
    })
}

/// A campaign file containing scenario files.
#[derive(Debug, Clone)]
pub struct Campaign<R>
where
    R: Read + Seek,
{
    reader: R,
    header: CampaignHeader,
    entries: Vec<ScenarioMeta>,
}

impl<R> Campaign<R>
where
    R: Read + Seek,
{
    /// Create a campaign instance from a readable input.
    ///
    /// This immediately reads the campaign header and scenario metadata, but not the scenario
    /// files themselves.
    pub fn from(mut input: R) -> Result<Self> {
        let header = read_campaign_header(&mut input)?;
        let mut entries = vec![];
        for _ in 0..header.num_scenarios {
            entries.push(read_scenario_meta(&mut input)?);
        }

        Ok(Self {
            reader: input,
            header,
            entries,
        })
    }

    /// Consume this Campaign instance and get the reader.
    pub fn into_inner(self) -> R {
        self.reader
    }

    /// Get the campaign file version.
    pub fn version(&self) -> CPXVersion {
        self.header.version
    }

    pub fn name(&self) -> &str {
        &self.header.name
    }

    /// Iterate over the scenario metadata for this campaign.
    pub fn entries(&self) -> impl Iterator<Item = &ScenarioMeta> {
        self.entries.iter()
    }

    /// Get the number of scenarios in this campaign.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get_name(&self, id: usize) -> Option<&str> {
        self.entries.get(id).map(|entry| entry.name.as_ref())
    }

    pub fn get_filename(&self, id: usize) -> Option<&str> {
        self.entries.get(id).map(|entry| entry.filename.as_ref())
    }

    fn get_id(&self, filename: &str) -> Option<usize> {
        self.entries
            .iter()
            .position(|entry| entry.filename == filename)
    }

    /// Get a scenario by its file name.
    pub fn by_name(&mut self, filename: &str) -> Result<Scenario> {
        self.by_name_raw(filename)
            .map(Cursor::new)
            .and_then(|mut buf| Scenario::from(&mut buf).map_err(ReadCampaignError::ParseSCXError))
    }

    /// Get a scenario by its campaign index.
    pub fn by_index(&mut self, index: usize) -> Result<Scenario> {
        self.by_index_raw(index)
            .map(Cursor::new)
            .and_then(|mut buf| Scenario::from(&mut buf).map_err(ReadCampaignError::ParseSCXError))
    }

    /// Get a scenario file buffer by its file name.
    pub fn by_name_raw(&mut self, filename: &str) -> Result<Vec<u8>> {
        self.get_id(filename)
            .ok_or(ReadCampaignError::NotFoundError)
            .and_then(|index| self.by_index_raw(index))
    }

    /// Get a scenario file buffer by its campaign index.
    pub fn by_index_raw(&mut self, index: usize) -> Result<Vec<u8>> {
        let entry = match self.entries.get(index) {
            Some(entry) => entry,
            None => return Err(ReadCampaignError::NotFoundError),
        };

        let mut result = vec![];

        self.reader.seek(SeekFrom::Start(entry.offset as u64))?;
        self.reader
            .by_ref()
            .take(entry.size as u64)
            .read_to_end(&mut result)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    /// Try to parse a file with an encoding that is not compatible with UTF-8.
    /// Source: http://aok.heavengames.com/blacksmith/showfile.php?fileid=884
    #[test]
    fn detect_encoding() {
        let f = File::open("./test/campaigns/DER FALL VON SACSAHUAMAN - TEIL I.cpx").unwrap();
        let cpx = Campaign::from(f).unwrap();
        assert_eq!(cpx.version(), *b"1.00");
        assert_eq!(cpx.name(), "DER FALL VON SACSAHUAMÁN - TEIL I");
        assert_eq!(cpx.len(), 1);

        let names: Vec<&String> = cpx.entries().map(|e| &e.name).collect();
        assert_eq!(names, vec!["Der Weg nach Sacsahuamán"]);
        let filenames: Vec<&String> = cpx.entries().map(|e| &e.filename).collect();
        assert_eq!(filenames, vec!["Der Weg nach Sacsahuamán.scx"]);
    }

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=1678
    #[test]
    fn aoe1_trial_cpn() {
        let f = File::open("test/campaigns/Armies at War A Combat Showcase.cpn").unwrap();
        let mut c = Campaign::from(f).expect("could not read meta");

        assert_eq!(c.version(), *b"1.00");
        assert_eq!(c.name(), "Armies at War, A Combat Showcase");
        assert_eq!(c.len(), 1);
        let names: Vec<&String> = c.entries().map(|e| &e.name).collect();
        assert_eq!(names, vec!["Bronze Age Art of War"]);
        let filenames: Vec<&String> = c.entries().map(|e| &e.filename).collect();
        assert_eq!(filenames, vec!["Bronze Age Art of War.scn"]);

        c.by_index_raw(0).expect("could not read raw file");
        c.by_name_raw("Bronze Age Art of War.scn")
            .expect("could not read raw file");
    }

    #[test]
    fn aoe1_beta_cpn() {
        let f = File::open("test/campaigns/Rise of Egypt Learning Campaign.cpn").unwrap();
        let c = Campaign::from(f).expect("could not read meta");

        assert_eq!(c.version(), *b"1.00");
        assert_eq!(c.name(), "Rise of Egypt Learning Campaign");
        assert_eq!(c.len(), 12);
        let filenames: Vec<&String> = c.entries().map(|e| &e.filename).collect();
        assert_eq!(
            filenames,
            vec![
                "HUNTING.scn",
                "FORAGING.scn",
                "Discoveries.scn",
                "Dawn of a New Age.scn",
                "SKIRMISH.scn",
                "Lands Unknown.scn",
                "FARMING.scn",
                "TRADE.scn",
                "CRUSADE.scn",
                "Establish a Second Colony.scn",
                "Naval Battle.scn",
                "Siege Battle.scn",
            ]
        );
    }
}
