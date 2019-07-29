use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Result, Write};

#[derive(Debug, Default, Clone)]
pub struct EffectCommand {
    pub command_type: u8,
    pub params: (i16, i16, i16, f32),
}

#[derive(Debug, Default, Clone)]
pub struct TechEffect {
    pub name: String,
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
        let mut name = [0; 31];
        input.read_exact(&mut name)?;
        let name =
            String::from_utf8(name.iter().cloned().take_while(|b| *b != 0).collect()).unwrap();

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
