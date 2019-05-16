use crate::{CampaignHeader, ScenarioMeta};
use byteorder::{WriteBytesExt, LE};
use genie_scx::Scenario;
use std::io::{Result, Write};

fn write_campaign_header<W: Write>(header: &CampaignHeader, output: &mut W) -> Result<()> {
    assert!(header.num_scenarios < std::i32::MAX as usize);

    output.write_all(&header.version)?;
    let mut name_bytes = header.name.as_bytes().to_vec();
    assert!(name_bytes.len() < 256);
    name_bytes.extend(vec![0; 256 - name_bytes.len()]);
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
    name_bytes.extend(vec![0; 255 - name_bytes.len()]);
    output.write_all(&name_bytes)?;

    let mut filename_bytes = meta.filename.as_bytes().to_vec();
    assert!(filename_bytes.len() < 255);
    filename_bytes.extend(vec![0; 255 - filename_bytes.len()]);
    output.write_all(&filename_bytes)?;

    Ok(())
}

struct CampaignEntry {
    name: String,
    filename: String,
    bytes: Vec<u8>,
}

impl CampaignEntry {
    fn name(&self) -> &str {
        &self.name
    }

    fn filename(&self) -> &str {
        &self.filename
    }

    fn size(&self) -> usize {
        self.bytes.len()
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub struct CampaignWriter<W: Write> {
    writer: W,
    header: CampaignHeader,
    scenarios: Vec<CampaignEntry>,
}

impl<W: Write> CampaignWriter<W> {
    pub fn new(name: &str, writer: W) -> Self {
        Self {
            writer,
            header: CampaignHeader::new(name),
            scenarios: vec![],
        }
    }

    pub fn add_raw(&mut self, name: &str, filename: &str, scx: Vec<u8>) {
        self.scenarios.push(CampaignEntry {
            name: name.to_owned(),
            filename: filename.to_owned(),
            bytes: scx,
        });
    }

    pub fn add(&mut self, name: &str, scx: &Scenario) -> Result<()> {
        let mut bytes = vec![];
        scx.write_to(&mut bytes)?;
        self.scenarios.push(CampaignEntry {
            name: name.to_owned(),
            filename: scx.filename().to_owned(),
            bytes,
        });
        Ok(())
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    fn write_header(&mut self) -> Result<()> {
        self.header.num_scenarios = self.scenarios.len();
        write_campaign_header(&self.header, &mut self.writer)
    }

    fn write_metas(&mut self) -> Result<()> {
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

    fn write_scenarios(&mut self) -> Result<()> {
        for scen in &self.scenarios {
            self.writer.write_all(scen.bytes())?;
        }
        Ok(())
    }

    pub fn flush(mut self) -> Result<W> {
        self.write_header()?;
        self.write_metas()?;
        self.write_scenarios()?;

        Ok(self.into_inner())
    }
}
