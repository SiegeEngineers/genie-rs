use crate::FileVersion;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{fallible_try_from, fallible_try_into, infallible_try_into};
use std::{
    convert::TryInto,
    io::{Read, Result, Write},
};

/// An ID identifying a sound.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SoundID(u16);
impl From<u16> for SoundID {
    fn from(n: u16) -> Self {
        SoundID(n)
    }
}

impl From<SoundID> for u16 {
    fn from(n: SoundID) -> Self {
        n.0
    }
}

impl From<SoundID> for usize {
    fn from(n: SoundID) -> Self {
        n.0.into()
    }
}

fallible_try_into!(SoundID, i16);
infallible_try_into!(SoundID, i32);
infallible_try_into!(SoundID, u32);
fallible_try_from!(SoundID, i16);
fallible_try_from!(SoundID, i32);

/// A "conceptual" sound, consisting of one or a group of sound files.
///
/// Items can be picked depending on the player's civilization, and depending on the probabilities
/// for each file.
#[derive(Debug, Default, Clone)]
pub struct Sound {
    /// Unique ID for this sound.
    pub id: SoundID,
    /// TODO document.
    pub play_delay: i16,
    /// TODO document.
    pub cache_time: i32,
    /// List of sound files in this sound.
    pub items: Vec<SoundItem>,
}

/// A single sound file.
#[derive(Debug, Default, Clone)]
pub struct SoundItem {
    /// Internal file name for this sound file.
    pub filename: String,
    /// DRS file ID for this sound file.
    pub resource_id: i32,
    /// The probability out of 100% that this file will be used for any given playback.
    pub probability: i16,
    /// Use this file for this civilization ID only.
    pub civilization: Option<i16>,
    /// File icon set (TODO what does this do?)
    pub icon_set: Option<i16>,
}

impl SoundItem {
    /// Read this sound item from an input stream.
    pub fn from<R: Read>(input: &mut R, _version: FileVersion) -> Result<Self> {
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

    /// Write this sound item to an input stream.
    pub fn write_to<W: Write>(&self, output: &mut W, _version: FileVersion) -> Result<()> {
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

impl Sound {
    /// Read this sound from an input stream.
    pub fn from<R: Read>(input: &mut R, version: FileVersion) -> Result<Self> {
        let mut sound = Sound::default();
        sound.id = input.read_u16::<LE>()?.into();
        sound.play_delay = input.read_i16::<LE>()?;
        let num_items = input.read_u16::<LE>()?;
        sound.cache_time = input.read_i32::<LE>()?;
        for _ in 0..num_items {
            sound.items.push(SoundItem::from(input, version)?);
        }
        Ok(sound)
    }

    /// Write this sound to an input stream.
    pub fn write_to<W: Write>(&self, output: &mut W, version: FileVersion) -> Result<()> {
        output.write_u16::<LE>(self.id.into())?;
        output.write_i16::<LE>(self.play_delay)?;
        output.write_u16::<LE>(self.len().try_into().unwrap())?;
        output.write_i32::<LE>(self.cache_time)?;
        for item in &self.items {
            item.write_to(output, version)?;
        }
        Ok(())
    }

    /// Get the number of sound files that are part of this "conceptual" sound.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if there are no sound files.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
