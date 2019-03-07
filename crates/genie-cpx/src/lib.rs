use std::io::{Seek, SeekFrom, Read, Write, Result, Error, ErrorKind};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_scx::Scenario;

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

#[derive(Debug, Clone)]
pub struct CampaignHeader {
    version: f32,
    name: String,
    num_scenarios: i32,
}

impl CampaignHeader {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let version = input.read_f32::<LE>()?;
        let name = read_fixed_str(input)?.expect("must have a name");
        let num_scenarios = input.read_i32::<LE>()?;

        Ok(CampaignHeader {
            version,
            name,
            num_scenarios,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_f32::<LE>(self.version)?;
        let mut name_bytes = self.name.as_bytes().to_vec();
        name_bytes.extend(vec![0; name_bytes.len() - 255]);
        output.write_all(&name_bytes)?;
        output.write_i32::<LE>(self.num_scenarios)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ScenarioMeta {
    size: i32,
    offset: i32,
    name: String,
    filename: String,
}

impl ScenarioMeta {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let size = input.read_i32::<LE>()?;
        let offset = input.read_i32::<LE>()?;
        let name = read_fixed_str(input)?.expect("must have a name");
        let filename = read_fixed_str(input)?.expect("must have a name");

        Ok(ScenarioMeta { size, offset,name,filename })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.size)?;
        output.write_i32::<LE>(self.offset)?;

        let mut name_bytes = self.name.as_bytes().to_vec();
        name_bytes.extend(vec![0; name_bytes.len() - 255]);
        output.write_all(&name_bytes)?;

        let mut filename_bytes = self.filename.as_bytes().to_vec();
        filename_bytes.extend(vec![0; filename_bytes.len() - 255]);
        output.write_all(&filename_bytes)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Campaign<R>
    where R: Read + Seek
{
    reader: R,
    header: CampaignHeader,
    entries: Vec<ScenarioMeta>,
    embedded_data: Vec<Option<Vec<u8>>>,
}

impl<R> Campaign<R>
    where R: Read + Seek
{
    pub fn from(mut input: R) -> Result<Self> {
        let header = CampaignHeader::from(&mut input)?;
        let mut entries = vec![];
        for _ in 0..header.num_scenarios {
            entries.push(ScenarioMeta::from(&mut input)?);
        }
        let embedded_data = vec![None; header.num_scenarios as usize];

        Ok(Self {
            reader: input,
            header,
            entries,
            embedded_data,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        assert_eq!(self.header.num_scenarios as usize, self.entries.len());
        assert_eq!(self.header.num_scenarios as usize, self.embedded_data.len());
        unimplemented!()
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

    pub fn add_scenario(&mut self, name: &str, scen: &Scenario) -> Result<()> {
        let mut bytes = vec![];
        scen.write_to(&mut bytes)?;
        self.add_raw(name, scen.filename(), &bytes)
    }

    pub fn add_raw(&mut self, name: &str, filename: &str, bytes: &[u8]) -> Result<()> {
        self.entries.push(ScenarioMeta {
            size: bytes.len() as i32,
            offset: 0,
            name: name.to_string(),
            filename: filename.to_string(),
        });
        self.embedded_data.push(Some(bytes.to_vec()));
        Ok(())
    }

    fn get_id(&self, filename: &str) -> Option<usize> {
        self.entries.iter().position(|entry| entry.filename == filename)
    }

    /// Get a scenario file buffer by its file name (really inefficiently by copying lots of stuff).
    pub fn by_name(&mut self, filename: &str) -> Result<Vec<u8>> {
        self.get_id(filename)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "scenario not found in campaign"))
            .and_then(|index| self.by_index(index))
    }

    /// Get a scenario file buffer by its campaign index (really inefficiently by copying lots of stuff).
    pub fn by_index(&mut self, index: usize) -> Result<Vec<u8>> {
        match self.embedded_data.get(index) {
            Some(Some(data)) => return Ok(data.to_vec()),
            _ => {},
        };

        let entry = match self.entries.get(index) {
            Some(entry) => entry,
            None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "scenario not found in campaign")),
        };

        let mut result = vec![];

        self.reader.seek(SeekFrom::Start(entry.offset as u64))?;
        (self.reader.by_ref() as &mut Read)
            .take(entry.size as u64)
            .read_to_end(&mut result);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
