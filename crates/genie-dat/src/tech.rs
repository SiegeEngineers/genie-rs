use arraystring::{ArrayString, typenum::U30};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Result, Write};

/// An effect command specifies an attribute change when a tech effect is triggered.
#[derive(Debug, Default, Clone)]
pub struct EffectCommand {
    /// The command.
    pub command_type: u8,
    /// Command-dependent parameters.
    pub params: (i16, i16, i16, f32),
}

pub type TechEffectName = ArrayString<U30>;

/// A tech effect is a group of attribute changes that are applied when the effect is triggered.
#[derive(Debug, Default, Clone)]
pub struct TechEffect {
    /// Name for the effect.
    pub name: TechEffectName,
    /// Attribute commands to execute when this effect is triggered.
    pub commands: Vec<EffectCommand>,
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
