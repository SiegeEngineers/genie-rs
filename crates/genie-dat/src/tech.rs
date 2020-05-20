use crate::civ::CivilizationID;
use crate::unit_type::UnitTypeID;
use arrayvec::{ArrayString, ArrayVec};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use encoding_rs::WINDOWS_1252;
pub use genie_support::TechID;
use genie_support::{read_opt_u16, read_opt_u32, MapInto, StringKey};
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
    required_techs: ArrayVec<[TechID; 6]>,
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

    /// Set the name of this effect.
    ///
    /// # Panics
    /// This function panics if `name` requires more than 31 bytes of storage.
    pub fn set_name(&mut self, name: &str) {
        self.name = TechEffectName::from(name).unwrap();
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
        let mut buffer = [0; 31];
        (&mut buffer[..self.name.len()]).copy_from_slice(self.name.as_bytes());
        output.write_all(&buffer)?;

        output.write_u16::<LE>(self.commands.len() as u16)?;
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

    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut tech = Self::default();
        for _ in 0..6 {
            // 4 on some versions
            if let Some(tech_id) = read_opt_u16(&mut input)? {
                tech.required_techs.push(tech_id);
            }
        }
        for _ in 0..3 {
            let effect = TechEffectRef::read_from(&mut input)?;
            if effect.effect_type != 0xFFFF {
                tech.effects.push(effect);
            }
        }
        let _num_required_techs = input.read_u16::<LE>()?;
        tech.civilization_id = read_opt_u16(&mut input)?;
        tech.full_tech_mode = input.read_u16::<LE>()?;
        tech.location = read_opt_u16(&mut input)?;
        tech.language_dll_name = read_opt_u16(&mut input)?;
        tech.language_dll_description = read_opt_u16(&mut input)?;
        tech.time = input.read_u16::<LE>()?;
        tech.time2 = input.read_u16::<LE>()?;
        tech.type_ = input.read_u16::<LE>()?;
        tech.icon_id = read_opt_u16(&mut input)?;
        tech.button_id = input.read_u8()?;
        tech.language_dll_help = read_opt_u32(&mut input)?;
        tech.help_page_id = input.read_u32::<LE>()?;
        tech.hotkey = read_opt_u32(&mut input)?;
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

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        for i in 0..6 {
            match self.required_techs.get(i) {
                Some(&id) => output.write_u16::<LE>(id.into())?,
                None => output.write_i16::<LE>(-1)?,
            }
        }
        for i in 0..3 {
            match self.effects.get(i) {
                Some(effect) => effect.write_to(&mut output)?,
                None => TechEffectRef {
                    effect_type: 0xFFFF,
                    amount: 0,
                    enabled: false,
                }
                .write_to(&mut output)?,
            }
        }
        output.write_u16::<LE>(self.required_techs.len() as u16)?;
        output.write_u16::<LE>(self.civilization_id.map_into().unwrap_or(0xFFFF))?;
        output.write_u16::<LE>(self.full_tech_mode)?;
        output.write_u16::<LE>(self.location.map_into().unwrap_or(0xFFFF))?;
        output.write_u16::<LE>(match self.language_dll_name {
            Some(StringKey::Num(id)) => id as u16,
            Some(_) => unreachable!("cannot use named string keys in dat files"),
            None => 0xFFFF,
        })?;
        output.write_u16::<LE>(match self.language_dll_description {
            Some(StringKey::Num(id)) => id as u16,
            Some(_) => unreachable!("cannot use named string keys in dat files"),
            None => 0xFFFF,
        })?;
        output.write_u16::<LE>(self.time)?;
        output.write_u16::<LE>(self.time2)?;
        output.write_u16::<LE>(self.type_)?;
        output.write_u16::<LE>(self.icon_id.map_into().unwrap_or(0xFFFF))?;
        output.write_u8(self.button_id)?;
        output.write_u32::<LE>(match self.language_dll_help {
            Some(StringKey::Num(id)) => id,
            Some(_) => unreachable!("cannot use named string keys in dat files"),
            None => 0xFFFF_FFFF,
        })?;
        output.write_u32::<LE>(self.help_page_id)?;
        output.write_u32::<LE>(self.hotkey.map_into().unwrap_or(0xFFFF_FFFF))?;
        let (encoded, _encoding, _failed) = WINDOWS_1252.encode(&self.name);
        output.write_u16::<LE>(encoded.len() as u16)?;
        output.write_all(encoded.as_ref())?;
        Ok(())
    }
}
