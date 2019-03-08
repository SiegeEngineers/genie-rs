use std::io::{Seek, SeekFrom, Read, Result, Error, ErrorKind};
use byteorder::{ReadBytesExt, LE};
use genie_scx::Scenario;
use crate::{CampaignHeader, ScenarioMeta};

pub fn read_fixed_str<R: Read>(input: &mut R) -> Result<Option<String>> {
    let mut bytes = [0; 255];
    input.read_exact(&mut bytes)?;

    let mut bytes = bytes.to_vec();
    if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
        bytes.truncate(end);
    }
    if bytes.is_empty() {
        Ok(None)
    } else {
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| Error::new(ErrorKind::Other, "invalid string"))
    }
}

fn read_campaign_header<R: Read>(input: &mut R) -> Result<CampaignHeader> {
    let version = input.read_f32::<LE>()?;
    let name = read_fixed_str(input)?.expect("must have a name");
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
    let name = read_fixed_str(input)?.expect("must have a name");
    let filename = read_fixed_str(input)?.expect("must have a name");

    Ok(ScenarioMeta {
        size,
        offset,
        name,
        filename,
    })
}

#[derive(Debug, Clone)]
pub struct Campaign<R>
    where R: Read + Seek
{
    reader: R,
    header: CampaignHeader,
    entries: Vec<ScenarioMeta>,
}

impl<R> Campaign<R>
    where R: Read + Seek
{
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

    pub fn into_inner(self) -> R {
        self.reader
    }

    pub fn version(&self) -> f32 {
        self.header.version
    }

    pub fn entries(&self) -> impl Iterator<Item = &ScenarioMeta> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    fn get_id(&self, filename: &str) -> Option<usize> {
        self.entries.iter().position(|entry| entry.filename == filename)
    }

    /// Get a scenario file buffer by its file name.
    pub fn by_name(&mut self, filename: &str) -> Result<Vec<u8>> {
        self.get_id(filename)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "scenario not found in campaign"))
            .and_then(|index| self.by_index(index))
    }

    /// Get a scenario file buffer by its campaign index.
    pub fn by_index(&mut self, index: usize) -> Result<Vec<u8>> {
        let entry = match self.entries.get(index) {
            Some(entry) => entry,
            None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "scenario not found in campaign")),
        };

        let mut result = vec![];

        self.reader.seek(SeekFrom::Start(entry.offset as u64))?;
        self.reader.by_ref()
            .take(entry.size as u64)
            .read_to_end(&mut result)?;

        Ok(result)
    }
}
