use crate::sound::SoundID;
use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{
    fallible_try_from, fallible_try_into, infallible_try_into, read_opt_u16, MapInto,
};
use std::convert::{TryFrom, TryInto};
use std::io::{Read, Result, Write};
use std::num::TryFromIntError;

/// An ID identifying a sprite.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SpriteID(u16);
impl From<u16> for SpriteID {
    fn from(n: u16) -> Self {
        SpriteID(n)
    }
}

impl From<SpriteID> for u16 {
    fn from(n: SpriteID) -> Self {
        n.0
    }
}

impl From<SpriteID> for i32 {
    fn from(n: SpriteID) -> Self {
        n.0.into()
    }
}

impl From<SpriteID> for u32 {
    fn from(n: SpriteID) -> Self {
        n.0.into()
    }
}

impl From<SpriteID> for usize {
    fn from(n: SpriteID) -> Self {
        n.0.into()
    }
}

fallible_try_into!(SpriteID, i16);
fallible_try_from!(SpriteID, i32);
fallible_try_from!(SpriteID, u32);

/// An ID identifying a string resource.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GraphicID(u32);

impl From<u16> for GraphicID {
    fn from(n: u16) -> Self {
        GraphicID(n.into())
    }
}

impl From<u32> for GraphicID {
    fn from(n: u32) -> Self {
        GraphicID(n)
    }
}

impl TryFrom<i32> for GraphicID {
    type Error = TryFromIntError;
    fn try_from(n: i32) -> std::result::Result<Self, Self::Error> {
        Ok(GraphicID(n.try_into()?))
    }
}

fallible_try_into!(GraphicID, i16);
infallible_try_into!(GraphicID, u32);
fallible_try_into!(GraphicID, i32);

#[derive(Debug, Default, Clone)]
pub struct SpriteDelta {
    pub sprite_id: Option<SpriteID>,
    pub offset_x: i16,
    pub offset_y: i16,
    pub display_angle: i16,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SoundProp {
    pub sound_delay: i16,
    pub sound_id: SoundID,
}

#[derive(Debug, Default, Clone)]
pub struct SpriteAttackSound {
    pub sound_props: ArrayVec<[SoundProp; 3]>,
}

#[derive(Debug, Default, Clone)]
pub struct Sprite {
    pub id: SpriteID,
    pub name: String,
    pub filename: String,
    pub slp_id: Option<GraphicID>,
    pub is_loaded: bool,
    color_flag: bool,
    pub layer: u8,
    pub color_table: u16,
    pub transparent_selection: bool,
    pub bounding_box: (i16, i16, i16, i16),
    pub sound_id: Option<SoundID>,
    pub num_frames: u16,
    num_facets: u16,
    pub base_speed: f32,
    pub frame_rate: f32,
    pub replay_delay: f32,
    pub sequence_type: i8,
    pub mirror_flag: i8,
    /// editor flag?
    other_flag: i8,
    pub deltas: Vec<SpriteDelta>,
    pub attack_sounds: Vec<SpriteAttackSound>,
}

impl SpriteDelta {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut delta = SpriteDelta::default();
        delta.sprite_id = read_opt_u16(input)?.map_into();
        let _padding = input.read_i16::<LE>()?;
        let _parent_sprite_pointer = input.read_i32::<LE>()?;
        delta.offset_x = input.read_i16::<LE>()?;
        delta.offset_y = input.read_i16::<LE>()?;
        delta.display_angle = input.read_i16::<LE>()?;
        let _padding = input.read_i16::<LE>()?;

        Ok(delta)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i16::<LE>(self.sprite_id.map(|v| v.try_into().unwrap()).unwrap_or(-1))?;
        // padding
        output.write_i16::<LE>(0)?;
        // pointer address to the parent sprite (overridden at load time by the game)
        output.write_i32::<LE>(0)?;
        output.write_i16::<LE>(self.offset_x)?;
        output.write_i16::<LE>(self.offset_y)?;
        output.write_i16::<LE>(self.display_angle)?;
        // padding
        output.write_i16::<LE>(0)?;

        Ok(())
    }
}

impl SoundProp {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let sound_delay = input.read_i16::<LE>()?;
        let sound_id = input.read_u16::<LE>()?.into();
        Ok(Self {
            sound_delay,
            sound_id,
        })
    }

    pub fn write_to<W: Write>(self, output: &mut W) -> Result<()> {
        output.write_i16::<LE>(self.sound_delay)?;
        output.write_u16::<LE>(self.sound_id.into())?;
        Ok(())
    }
}

impl SpriteAttackSound {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut val = SpriteAttackSound::default();
        for _ in 0..val.sound_props.capacity() {
            let prop = SoundProp::read_from(input)?;
            if u16::from(prop.sound_id) != 0xFFFF {
                val.sound_props.push(prop);
            }
        }
        Ok(val)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        for index in 0..self.sound_props.capacity() {
            let prop = self.sound_props.get(index).cloned().unwrap_or_default();
            prop.write_to(output)?;
        }
        Ok(())
    }
}

impl Sprite {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut sprite = Sprite::default();
        let mut name = [0u8; 21];
        input.read_exact(&mut name)?;
        sprite.name =
            String::from_utf8(name.iter().cloned().take_while(|b| *b != 0).collect()).unwrap();
        let mut filename = [0u8; 13];
        input.read_exact(&mut filename)?;
        sprite.filename =
            String::from_utf8(filename.iter().cloned().take_while(|b| *b != 0).collect()).unwrap();
        sprite.slp_id = {
            let num = input.read_i32::<LE>()?;
            if num == -1 {
                None
            } else {
                Some(num.try_into().unwrap())
            }
        };
        sprite.is_loaded = input.read_u8()? != 0;
        sprite.color_flag = input.read_u8()? != 0;
        sprite.layer = input.read_u8()?;
        sprite.color_table = input.read_u16::<LE>()?;
        sprite.transparent_selection = input.read_u8()? != 0;
        sprite.bounding_box = (
            input.read_i16::<LE>()?,
            input.read_i16::<LE>()?,
            input.read_i16::<LE>()?,
            input.read_i16::<LE>()?,
        );
        let num_deltas = input.read_u16::<LE>()?;
        sprite.sound_id = read_opt_u16(input)?.map_into();
        let attack_sounds_used = input.read_u8()? != 0;
        sprite.num_frames = input.read_u16::<LE>()?;
        sprite.num_facets = input.read_u16::<LE>()?;
        sprite.base_speed = input.read_f32::<LE>()?;
        sprite.frame_rate = input.read_f32::<LE>()?;
        sprite.replay_delay = input.read_f32::<LE>()?;
        sprite.sequence_type = input.read_i8()?;
        sprite.id = input.read_u16::<LE>()?.into();
        sprite.mirror_flag = input.read_i8()?;
        sprite.other_flag = input.read_i8()?;

        for _ in 0..num_deltas {
            sprite.deltas.push(SpriteDelta::read_from(input)?);
        }
        if attack_sounds_used {
            for _ in 0..sprite.num_facets {
                sprite
                    .attack_sounds
                    .push(SpriteAttackSound::read_from(input)?);
            }
        }

        Ok(sprite)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        if !self.attack_sounds.is_empty() {
            assert_eq!(self.attack_sounds.len(), usize::from(self.num_facets));
        }
        let mut name = [0u8; 21];
        (&mut name[..]).write_all(self.name.as_bytes())?;
        let mut filename = [0u8; 13];
        (&mut filename[..]).write_all(self.filename.as_bytes())?;
        output.write_i32::<LE>(self.slp_id.map(|v| v.try_into().unwrap()).unwrap_or(-1))?;
        output.write_u8(if self.is_loaded { 1 } else { 0 })?;
        output.write_u8(if self.color_flag { 1 } else { 0 })?;
        output.write_u8(self.layer)?;
        output.write_u16::<LE>(self.color_table)?;
        output.write_u8(if self.transparent_selection { 1 } else { 0 })?;
        output.write_i16::<LE>(self.bounding_box.0)?;
        output.write_i16::<LE>(self.bounding_box.1)?;
        output.write_i16::<LE>(self.bounding_box.2)?;
        output.write_i16::<LE>(self.bounding_box.3)?;

        output.write_u16::<LE>(self.deltas.len().try_into().unwrap())?;
        output.write_i16::<LE>(self.sound_id.map(|v| v.try_into().unwrap()).unwrap_or(-1))?;
        output.write_u8(if self.attack_sounds.is_empty() { 0 } else { 1 })?;
        output.write_u16::<LE>(self.num_frames)?;
        output.write_u16::<LE>(self.num_facets)?;
        output.write_f32::<LE>(self.base_speed)?;
        output.write_f32::<LE>(self.frame_rate)?;
        output.write_f32::<LE>(self.replay_delay)?;
        output.write_i8(self.sequence_type)?;
        output.write_u16::<LE>(self.id.into())?;
        output.write_i8(self.mirror_flag)?;
        output.write_i8(self.other_flag)?;

        for delta in &self.deltas {
            delta.write_to(output)?;
        }
        for sound in &self.attack_sounds {
            sound.write_to(output)?;
        }
        Ok(())
    }
}
