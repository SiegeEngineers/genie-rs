use crate::{unit_type::StringID, unit_type::UnitTypeID};
use arraystring::{typenum::U30, ArrayString};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::MapInto;
use std::{
    convert::TryInto,
    io::{self, Read, Result, Write},
};

/// An effect command specifies an attribute change when a tech effect is triggered.
#[derive(Debug, Default, Clone)]
pub struct EffectCommand {
    /// The command.
    pub command_type: u8,
    /// Command-dependent parameters.
    pub params: (i16, i16, i16, f32),
}

type TechEffectName = ArrayString<U30>;

/// A tech effect is a group of attribute changes that are applied when the effect is triggered.
#[derive(Debug, Default, Clone)]
pub struct TechEffect {
    /// Name for the effect.
    name: TechEffectName,
    /// Attribute commands to execute when this effect is triggered.
    pub commands: Vec<EffectCommand>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TechEffectRef {
    pub effect_type: u16,
    pub amount: u16,
    pub enabled: bool,
}

#[derive(Debug, Default, Clone)]
pub struct Tech {
    required_techs: [i16; 6],
    effects: [TechEffectRef; 3],
    civ_id: u16,
    full_tech_mode: u16,
    location: Option<UnitTypeID>,
    language_dll_name: Option<StringID>,
    language_dll_description: Option<StringID>,
    time: u16,
    time2: u16,
    type_: u16,
    icon_id: Option<u16>,
    button_id: u8,
    language_dll_help: Option<StringID>,
    help_page_id: u32,
    hotkey: Option<u32>,
    name: String,
}

impl Tech {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut tech = Self::default();
        for req in tech.required_techs.iter_mut() {
            *req = input.read_i16::<LE>()?;
        }
        for effect in tech.effects.iter_mut() {
            *effect = TechEffectRef::from(input)?;
        }
        let _num_required_techs = input.read_u16::<LE>()?;
        tech.civ_id = input.read_u16::<LE>()?;
        tech.full_tech_mode = input.read_u16::<LE>()?;
        tech.location = read_opt_u16(input)?.map_into();
        tech.language_dll_name = read_opt_u16(input)?.map_into();
        tech.language_dll_description = read_opt_u16(input)?.map_into();
        tech.time = input.read_u16::<LE>()?;
        tech.time2 = input.read_u16::<LE>()?;
        tech.type_ = input.read_u16::<LE>()?;
        tech.icon_id = read_opt_u16(input)?;
        tech.button_id = input.read_u8()?;
        tech.language_dll_help = {
            let n = input.read_i32::<LE>()?;
            if n < 0 {
                None
            } else {
                Some(n.try_into().unwrap())
            }
        };
        tech.help_page_id = input.read_u32::<LE>()?;
        tech.hotkey = {
            let n = input.read_i32::<LE>()?;
            if n < 0 { None } else { Some(n.try_into().unwrap()) }
        };
        tech.name = {
            let name_len = input.read_u16::<LE>()?;
            let mut bytes = vec![0; name_len as usize];
            input.read_exact(&mut bytes)?;
            String::from_utf8(bytes.iter().copied().take_while(|b| *b != 0).collect()).unwrap()
        };
        Ok(tech)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        Ok(())
    }
}

impl EffectCommand {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let command_type = input.read_u8()?;
        let params = (
            input.read_i16::<LE>()?,
            input.read_i16::<LE>()?,
            input.read_i16::<LE>()?,
            input.read_f32::<LE>()?,
        );
        Ok(EffectCommand {
            command_type,
            params,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.command_type)?;
        output.write_i16::<LE>(self.params.0)?;
        output.write_i16::<LE>(self.params.1)?;
        output.write_i16::<LE>(self.params.2)?;
        output.write_f32::<LE>(self.params.3)?;
        Ok(())
    }
}

impl TechEffect {
    /// Get the name of this effect.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let name = {
            let mut bytes = [0; 31];
            input.read_exact(&mut bytes)?;
            let bytes: Vec<u8> = bytes.iter().cloned().take_while(|b| *b != 0).collect();
            TechEffectName::from_utf8(bytes).unwrap()
        };

        let num_effects = input.read_u16::<LE>()?;
        let mut commands = vec![EffectCommand::default(); num_effects as usize];
        for effect in commands.iter_mut() {
            *effect = EffectCommand::from(input)?;
        }

        Ok(TechEffect { name, commands })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_all(&[0; 31])?;

        for effect in &self.commands {
            effect.write_to(output)?;
        }
        Ok(())
    }
}

impl TechEffectRef {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            effect_type: input.read_u16::<LE>()?,
            amount: input.read_u16::<LE>()?,
            enabled: input.read_u8()? != 0,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u16::<LE>(self.effect_type)?;
        output.write_u16::<LE>(self.amount)?;
        output.write_u8(if self.enabled { 1 } else { 0 })?;
        Ok(())
    }
}

fn read_opt_u16<R: Read>(input: &mut R) -> Result<Option<u16>> {
    let v = input.read_i16::<LE>()?;
    if v == -1 {
        return Ok(None);
    }
    Ok(Some(
        v.try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
    ))
}
