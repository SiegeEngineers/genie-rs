use crate::Version;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Result, Write};

#[derive(Debug, Default, Clone)]
pub struct SoundItem {
    pub filename: String,
    pub resource_id: i32,
    pub probability: i16,
    pub civilization: Option<i16>,
    pub icon_set: Option<i16>,
}

impl SoundItem {
    pub fn from<R: Read>(input: &mut R, _version: Version) -> Result<Self> {
        let mut item = SoundItem::default();
        let mut filename = [0u8; 13];
        input.read_exact(&mut filename)?;
        item.resource_id = input.read_i32::<LE>()?;
        item.probability = input.read_i16::<LE>()?;
        // AoK only
        item.civilization = Some(input.read_i16::<LE>()?);
        item.icon_set = Some(input.read_i16::<LE>()?);

        Ok(item)
    }

    pub fn write_to<W: Write>(&self, output: &mut W, _version: Version) -> Result<()> {
        output.write_i32::<LE>(self.resource_id)?;
        output.write_i16::<LE>(self.probability)?;
        // AoK only, must both be set
        assert!(self.civilization.is_some());
        assert!(self.icon_set.is_some());
        output.write_i16::<LE>(self.civilization.unwrap())?;
        output.write_i16::<LE>(self.icon_set.unwrap())?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Sound {
    pub id: i16,
    pub play_delay: i16,
    pub cache_time: i32,
    pub items: Vec<SoundItem>,
}

impl Sound {
    pub fn from<R: Read>(input: &mut R, version: Version) -> Result<Self> {
        let mut sound = Sound::default();
        sound.id = input.read_i16::<LE>()?;
        sound.play_delay = input.read_i16::<LE>()?;
        let num_items = input.read_u16::<LE>()?;
        sound.cache_time = input.read_i32::<LE>()?;
        for _ in 0..num_items {
            sound.items.push(SoundItem::from(input, version)?);
        }
        Ok(sound)
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: Version) -> Result<()> {
        output.write_i16::<LE>(self.id)?;
        output.write_i16::<LE>(self.play_delay)?;
        output.write_u16::<LE>(self.len() as u16)?;
        output.write_i32::<LE>(self.cache_time)?;
        for item in &self.items {
            item.write_to(output, version)?;
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
