use crate::{CPXVersion, CampaignHeader, ScenarioMeta, AOE1_DE, AOE2_DE};
use byteorder::{ReadBytesExt, LE};
use chardet::detect as detect_encoding;
use encoding_rs::Encoding;
use genie_scx::{self as scx, DLCPackage, Scenario};
use std::convert::TryFrom;
use std::io::{self, Cursor, Read, Seek, SeekFrom};

/// Type for errrors that could occur while reading/parsing a campaign file.
#[derive(Debug, thiserror::Error)]
pub enum ReadCampaignError {
    /// A string could not be decoded, its encoding may be unknown or it may be binary nonsense.
    #[error("invalid string")]
    DecodeStringError,
    /// An I/O error occurred.
    #[error("{}", .0)]
    IoError(#[from] io::Error),
    /// The campaign file or a scenario inside it is missing a user-facing name value.
    #[error("campaign or scenario must have a name")]
    MissingNameError,
    /// The requested scenario file does not exist in this campaign file.
    #[error("scenario not fonud in campaign")]
    NotFoundError,
    /// A scenario file could not be parsed.
    #[error("{}", .0)]
    ParseSCXError(#[from] scx::Error),
}

type Result<T> = std::result::Result<T, ReadCampaignError>;

/// Decode a string with unknown encoding.
fn decode_str(bytes: &[u8]) -> Result<String> {
    if bytes.is_empty() {
        return Ok("".to_string());
    }

    let (encoding_name, _confidence, _language) = detect_encoding(bytes);
    Encoding::for_label(encoding_name.as_bytes())
        .ok_or(ReadCampaignError::DecodeStringError)
        .and_then(|encoding| {
            let (decoded, _enc, failed) = encoding.decode(bytes);
            if failed {
                return Err(ReadCampaignError::DecodeStringError);
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

fn read_hd_or_later_string<R: Read>(input: &mut R) -> Result<Option<String>> {
    let open = input.read_u16::<LE>()?;
    // Check that this actually is the start of a string
    if open != 0x0A60 {
        return Err(ReadCampaignError::DecodeStringError);
    }
    let len = input.read_u16::<LE>()? as usize;
    let mut bytes = vec![0; len];
    input.read_exact(&mut bytes[0..len])?;
    decode_str(&bytes).map(Some)
}

fn read_campaign_header<R: Read>(input: &mut R) -> Result<CampaignHeader> {
    let mut version = [0; 4];
    input.read_exact(&mut version)?;

    let num_scenarios;
    let name;

    if version == AOE1_DE {
        num_scenarios = input.read_u32::<LE>()? as usize;
        name = read_hd_or_later_string(input)?.ok_or(ReadCampaignError::MissingNameError)?;
    } else {
        // DE2 added package dependency data. We don't store that right now, because DE always
        // supports all packages.[citation needed]
        if version == AOE2_DE {
            let num_dependencies = input.read_u32::<LE>()?;
            let mut dependencies = vec![DLCPackage::AgeOfKings; num_dependencies as usize];
            for dependency in dependencies.iter_mut() {
                *dependency =
                    DLCPackage::try_from(input.read_i32::<LE>()?).map_err(scx::Error::from)?;
            }
        }

        name = read_fixed_str(input, 256)?.ok_or(ReadCampaignError::MissingNameError)?;
        num_scenarios = input.read_u32::<LE>()? as usize;
    }

    Ok(CampaignHeader {
        version,
        name,
        num_scenarios,
    })
}

fn read_scenario_meta_de2<R: Read>(input: &mut R) -> Result<ScenarioMeta> {
    let size = input.read_u32::<LE>()? as usize;
    let offset = input.read_u32::<LE>()? as usize;
    let name = read_hd_or_later_string(input)?.ok_or(ReadCampaignError::MissingNameError)?;
    let filename = read_hd_or_later_string(input)?.ok_or(ReadCampaignError::MissingNameError)?;

    Ok(ScenarioMeta {
        size,
        offset,
        name,
        filename,
    })
}

fn read_scenario_meta_de<R: Read>(input: &mut R) -> Result<ScenarioMeta> {
    let size = input.read_u64::<LE>()? as usize;
    let offset = input.read_u64::<LE>()? as usize;
    let name = read_hd_or_later_string(input)?.ok_or(ReadCampaignError::MissingNameError)?;
    let filename = read_hd_or_later_string(input)?.ok_or(ReadCampaignError::MissingNameError)?;

    Ok(ScenarioMeta {
        size,
        offset,
        name,
        filename,
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
        let read_entry = if header.version == *b"2.00" {
            read_scenario_meta_de2
        } else if header.version == *b"1.10" {
            read_scenario_meta_de
        } else {
            read_scenario_meta
        };
        for _ in 0..header.num_scenarios {
            entries.push(read_entry(&mut input)?);
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

    /// Get the user-facing name of this campaign.
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

    /// Returns true if this campaign contains no scenario files.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the user-facing name of the scenario at the given index.
    pub fn get_name(&self, id: usize) -> Option<&str> {
        self.entries.get(id).map(|entry| entry.name.as_ref())
    }

    /// Get the filename of the scenario at the given index.
    pub fn get_filename(&self, id: usize) -> Option<&str> {
        self.entries.get(id).map(|entry| entry.filename.as_ref())
    }

    /// Return the index of the scenario with the given filename, if it exists.
    fn get_id(&self, filename: &str) -> Option<usize> {
        self.entries
            .iter()
            .position(|entry| entry.filename == filename)
    }

    /// Get a scenario by its file name.
    pub fn by_name(&mut self, filename: &str) -> Result<Scenario> {
        self.by_name_raw(filename)
            .map(Cursor::new)
            .and_then(|buf| Scenario::read_from(buf).map_err(ReadCampaignError::ParseSCXError))
    }

    /// Get a scenario by its campaign index.
    pub fn by_index(&mut self, index: usize) -> Result<Scenario> {
        self.by_index_raw(index)
            .map(Cursor::new)
            .and_then(|buf| Scenario::read_from(buf).map_err(ReadCampaignError::ParseSCXError))
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
    use crate::{AOE1_DE, AOE2_DE, AOE_AOK};
    use anyhow::Context;
    use std::fs::File;

    /// Try to parse a file with an encoding that is not compatible with UTF-8.
    /// Source: http://aok.heavengames.com/blacksmith/showfile.php?fileid=884
    #[test]
    fn detect_encoding() -> anyhow::Result<()> {
        let f = File::open("./test/campaigns/DER FALL VON SACSAHUAMAN - TEIL I.cpx")?;
        let cpx = Campaign::from(f)?;
        assert_eq!(cpx.version(), AOE_AOK);
        assert_eq!(cpx.name(), "DER FALL VON SACSAHUAMÁN - TEIL I");
        assert_eq!(cpx.len(), 1);

        let names: Vec<_> = cpx.entries().map(|e| &e.name).collect();
        assert_eq!(names, vec!["Der Weg nach Sacsahuamán"]);
        let filenames: Vec<_> = cpx.entries().map(|e| &e.filename).collect();
        assert_eq!(filenames, vec!["Der Weg nach Sacsahuamán.scx"]);
        Ok(())
    }

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=1678
    #[test]
    fn aoe1_trial_cpn() -> anyhow::Result<()> {
        let f = File::open("test/campaigns/Armies at War A Combat Showcase.cpn")?;
        let mut c = Campaign::from(f).context("could not read meta")?;

        assert_eq!(c.version(), AOE_AOK);
        assert_eq!(c.name(), "Armies at War, A Combat Showcase");
        assert_eq!(c.len(), 1);
        let names: Vec<_> = c.entries().map(|e| &e.name).collect();
        assert_eq!(names, vec!["Bronze Age Art of War"]);
        let filenames: Vec<_> = c.entries().map(|e| &e.filename).collect();
        assert_eq!(filenames, vec!["Bronze Age Art of War.scn"]);

        c.by_index_raw(0).context("could not read raw file")?;
        c.by_name_raw("Bronze Age Art of War.scn")
            .context("could not read raw file")?;
        Ok(())
    }

    #[test]
    fn aoe1_beta_cpn() -> anyhow::Result<()> {
        let f = File::open("test/campaigns/Rise of Egypt Learning Campaign.cpn")?;
        let c = Campaign::from(f).context("could not read meta")?;

        assert_eq!(c.version(), AOE_AOK);
        assert_eq!(c.name(), "Rise of Egypt Learning Campaign");
        assert_eq!(c.len(), 12);
        let filenames: Vec<_> = c.entries().map(|e| &e.filename).collect();
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
        Ok(())
    }

    #[test]
    fn aoe_de() -> anyhow::Result<()> {
        let f = File::open("test/campaigns/10 The First Punic War.aoecpn")?;
        let c = Campaign::from(f)?;

        assert_eq!(c.version(), AOE1_DE);
        assert_eq!(c.name(), "10 The First Punic War");
        assert_eq!(c.len(), 3);
        let filenames: Vec<_> = c.entries().map(|e| &e.filename).collect();
        assert_eq!(
            filenames,
            vec![
                "Scxt1-01-The Battle of Agrigentum.aoescn",
                "Scxt1-02-The Battle of Mylae.aoescn",
                "Scxt1-03-The Battle of Tunis.aoescn",
            ]
        );

        /* Enable when genie_scx supports DE1 scenarios better
        let mut c = c;
        for i in 0..c.len() {
            let _scen = c.by_index(i)?;
        }
        */

        Ok(())
    }

    #[test]
    fn aoe_de2() -> Result<()> {
        let f = File::open("test/campaigns/acam1.aoe2campaign")?;
        let c = Campaign::from(f)?;

        assert_eq!(c.version(), AOE2_DE);
        assert_eq!(c.name(), "acam1");
        assert_eq!(c.len(), 5);
        let filenames: Vec<_> = c.entries().map(|e| &e.filename).collect();
        assert_eq!(
            filenames,
            vec![
                "A1_Tariq1.aoe2scenario",
                "A1_Tariq2.aoe2scenario",
                "A1_Tariq3.aoe2scenario",
                "A1_Tariq4.aoe2scenario",
                "A1_Tariq5.aoe2scenario",
            ]
        );

        let f = File::open("test/campaigns/rcam3.aoe2campaign")?;
        let c = Campaign::from(f)?;

        assert_eq!(c.version(), AOE2_DE);
        assert_eq!(c.name(), "rcam3");
        assert_eq!(c.len(), 5);
        let filenames: Vec<_> = c.entries().map(|e| &e.filename).collect();
        assert_eq!(
            filenames,
            vec![
                "R3_Bayinnaung_1.aoe2scenario",
                "R3_Bayinnaung_2.aoe2scenario",
                "R3_Bayinnaung_3.aoe2scenario",
                "R3_Bayinnaung_4.aoe2scenario",
                "R3_Bayinnaung_5.aoe2scenario",
            ]
        );

        /* Enable when genie_scx supports DE2 scenarios better
        let mut c = c;
        for i in 0..c.len() {
            let _scen = c.by_index(i)?;
        }
        */

        Ok(())
    }
}
