use crate::civ::CivilizationID;
use crate::unit_type::UnitTypeID;
use arrayvec::{ArrayString, ArrayVec};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use encoding_rs::WINDOWS_1252;
pub use genie_support::TechID;
use genie_support::{read_opt_u16, MapInto, StringKey};
use std::convert::TryInto;
use std::io::{Read, Result, Write};

/// An effect command specifies an attribute change when a tech effect is triggered.
#[derive(Debug, Default, Clone)]
pub struct EffectCommand {
    /// The command.
    pub command_type: u8,
    /// Command-dependent parameters.
    pub params: (i16, i16, i16, f32),
}

type TechEffectName = ArrayString<[u8; 31]>;

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
    required_techs: ArrayVec<[i16; 6]>,
    effects: ArrayVec<[TechEffectRef; 3]>,
    civilization_id: Option<CivilizationID>,
    full_tech_mode: u16,
    location: Option<UnitTypeID>,
    language_dll_name: Option<StringKey>,
    language_dll_description: Option<StringKey>,
    time: u16,
    time2: u16,
    type_: u16,
    icon_id: Option<u16>,
    button_id: u8,
    language_dll_help: Option<StringKey>,
    help_page_id: u32,
    hotkey: Option<u32>,
    name: String,
}

impl EffectCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
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

    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut effect = Self::default();
        let mut bytes = [0; 31];
        input.read_exact(&mut bytes)?;
        let bytes = &bytes[..bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len())];
        let (name, _encoding, _failed) = WINDOWS_1252.decode(&bytes);
        effect.name = TechEffectName::from(&name).unwrap();

        let num_commands = input.read_u16::<LE>()?;
        for _ in 0..num_commands {
            effect.commands.push(EffectCommand::read_from(input)?);
        }

        Ok(effect)
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
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            effect_type: input.read_u16::<LE>()?,
            amount: input.read_u16::<LE>()?,
            enabled: input.read_u8()? != 0,
        })
    }

    pub fn write_to<W: Write>(self, output: &mut W) -> Result<()> {
        output.write_u16::<LE>(self.effect_type)?;
        output.write_u16::<LE>(self.amount)?;
        output.write_u8(if self.enabled { 1 } else { 0 })?;
        Ok(())
    }
}

impl Tech {
    /// Get the name of this tech.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut tech = Self::default();
        for _ in 0..6 {
            // 4 on some versions
            tech.required_techs.push(input.read_i16::<LE>()?);
        }
        for _ in 0..3 {
            let effect = TechEffectRef::read_from(input)?;
            if effect.effect_type != 0xFFFF {
                tech.effects.push(effect);
            }
        }
        let _num_required_techs = input.read_u16::<LE>()?;
        tech.civilization_id = match input.read_u16::<LE>()? {
            0xFFFF => None,
            n => Some(n.try_into().unwrap()),
        };
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
            if n < 0 {
                None
            } else {
                Some(n.try_into().unwrap())
            }
        };
        tech.name = {
            let name_len = input.read_u16::<LE>()?;
            let mut bytes = vec![0; name_len as usize];
            input.read_exact(&mut bytes)?;
            let bytes = &bytes[..bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len())];
            let (name, _encoding, _failed) = WINDOWS_1252.decode(&bytes);
            name.to_string()
        };
        Ok(tech)
    }

    pub fn write_to<W: Write>(&self, _output: &mut W) -> Result<()> {
        unimplemented!()
    }
}
