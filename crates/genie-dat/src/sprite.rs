use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Result, Write};

#[derive(Debug, Default, Clone)]
pub struct SpriteDelta {
    pub sprite_id: i16,
    pub offset_x: i16,
    pub offset_y: i16,
    pub display_angle: i16,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SoundProp {
    pub sound_delay: i16,
    pub sound_id: i16,
}

#[derive(Debug, Default)]
pub struct SpriteAttackSound {
    pub sound_props: [SoundProp; 3],
}

#[derive(Debug, Default)]
pub struct Sprite {
    pub id: u16,
    pub name: String,
    pub filename: String,
    pub slp_id: i32,
    pub is_loaded: bool,
    color_flag: bool,
    pub layer: u8,
    pub color_table: u16,
    pub transparent_selection: bool,
    pub bounding_box: (i16, i16, i16, i16),
    pub sound_id: Option<u16>,
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
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut delta = SpriteDelta::default();
        delta.sprite_id = input.read_i16::<LE>()?;
        // padding
        input.read_i16::<LE>()?;
        // pointer address to the parent sprite (overridden at load time by the game)
        input.read_i32::<LE>()?;
        delta.offset_x = input.read_i16::<LE>()?;
        delta.offset_y = input.read_i16::<LE>()?;
        delta.display_angle = input.read_i16::<LE>()?;
        // padding
        input.read_i16::<LE>()?;

        Ok(delta)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i16::<LE>(self.sprite_id)?;
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
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let sound_delay = input.read_i16::<LE>()?;
        let sound_id = input.read_i16::<LE>()?;
        Ok(Self {
            sound_delay,
            sound_id,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i16::<LE>(self.sound_delay)?;
        output.write_i16::<LE>(self.sound_id)?;
        Ok(())
    }
}

impl SpriteAttackSound {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut val = SpriteAttackSound::default();
        for prop in val.sound_props.iter_mut() {
            *prop = SoundProp::from(input)?;
        }
        Ok(val)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        for prop in &self.sound_props {
            prop.write_to(output)?;
        }
        Ok(())
    }
}

impl Sprite {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut sprite = Sprite::default();
        let mut name = [0u8; 21];
        input.read_exact(&mut name)?;
        sprite.name =
            String::from_utf8(name.iter().cloned().take_while(|b| *b != 0).collect()).unwrap();
        let mut filename = [0u8; 13];
        input.read_exact(&mut filename)?;
        sprite.filename =
            String::from_utf8(filename.iter().cloned().take_while(|b| *b != 0).collect()).unwrap();
        sprite.slp_id = input.read_i32::<LE>()?;
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
        sprite.sound_id = match input.read_i16::<LE>()? {
            -1 => None,
            id => Some(id as u16),
        };
        let attack_sounds_used = input.read_u8()? != 0;
        sprite.num_frames = input.read_u16::<LE>()?;
        sprite.num_facets = input.read_u16::<LE>()?;
        sprite.base_speed = input.read_f32::<LE>()?;
        sprite.frame_rate = input.read_f32::<LE>()?;
        sprite.replay_delay = input.read_f32::<LE>()?;
        sprite.sequence_type = input.read_i8()?;
        sprite.id = input.read_u16::<LE>()?;
        sprite.mirror_flag = input.read_i8()?;
        sprite.other_flag = input.read_i8()?;

        for _ in 0..num_deltas {
            sprite.deltas.push(SpriteDelta::from(input)?);
        }
        if attack_sounds_used {
            for _ in 0..sprite.num_facets {
                sprite.attack_sounds.push(SpriteAttackSound::from(input)?);
            }
        }

        Ok(sprite)
    }
}
