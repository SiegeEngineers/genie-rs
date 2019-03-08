use std::io::{Seek, SeekFrom, Write, Result};
use byteorder::{WriteBytesExt, LE};
use crate::{CampaignHeader, ScenarioMeta};

fn write_campaign_header<W: Write>(header: &CampaignHeader, output: &mut W) -> Result<()> {
    assert!(header.num_scenarios < std::i32::MAX as usize);

    output.write_f32::<LE>(header.version)?;
    let mut name_bytes = header.name.as_bytes().to_vec();
    assert!(name_bytes.len() < 255);
    name_bytes.extend(vec![0; name_bytes.len() - 255]);
    output.write_all(&name_bytes)?;
    output.write_i32::<LE>(header.num_scenarios as i32)?;
    Ok(())
}

fn write_scenario_meta<W: Write>(meta: &ScenarioMeta, output: &mut W) -> Result<()> {
    assert!(meta.size < std::i32::MAX as usize);
    assert!(meta.offset < std::i32::MAX as usize);

    output.write_i32::<LE>(meta.size as i32)?;
    output.write_i32::<LE>(meta.offset as i32)?;

    let mut name_bytes = meta.name.as_bytes().to_vec();
    assert!(name_bytes.len() < 255);
    name_bytes.extend(vec![0; name_bytes.len() - 255]);
    output.write_all(&name_bytes)?;

    let mut filename_bytes = meta.filename.as_bytes().to_vec();
    assert!(filename_bytes.len() < 255);
    filename_bytes.extend(vec![0; filename_bytes.len() - 255]);
    output.write_all(&filename_bytes)?;

    Ok(())
}

pub struct CampaignWriter<W>
    where W: Write + Seek
{
    writer: W,
    header: CampaignHeader,
    scenarios: Vec<ScenarioMeta>,
}

impl<W> CampaignWriter<W>
    where W: Write + Seek
{
    pub fn new(name: &str, writer: W) -> Self {
        Self {
            writer,
            header: CampaignHeader::new(name),
            scenarios: vec![],
        }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    fn write_header(&mut self) -> Result<()> {
        write_campaign_header(&self.header, &mut self.writer)
    }

    pub fn flush(mut self) -> Result<()> {
        self.writer.seek(SeekFrom::Start(0))?;
        self.write_header()
    }
}
