use crate::victory::VictoryConditions;
use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{write_opt_str, ReadStringsExt};
use std::io::{Read, Write};

#[derive(Debug, Default, Clone)]
pub struct PlayerBaseProperties {
    pub(crate) posture: i32,
    pub(crate) player_type: i32,
    pub(crate) civilization: i32,
    pub(crate) active: i32,
}

#[derive(Debug, Default, Clone)]
pub struct PlayerFiles {
    /// Obsolete.
    pub(crate) build_list: Option<String>,
    /// Obsolete.
    pub(crate) city_plan: Option<String>,
    /// String content of the AI of this player.
    pub(crate) ai_rules: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlayerStartResources {
    pub(crate) gold: i32,
    pub(crate) wood: i32,
    pub(crate) food: i32,
    pub(crate) stone: i32,
    pub(crate) ore: i32,
    pub(crate) goods: i32,
    pub(crate) player_color: Option<i32>,
}

impl Default for PlayerStartResources {
    fn default() -> Self {
        Self {
            gold: 100,
            wood: 200,
            food: 200,
            stone: 200,
            ore: 100,
            goods: 0,
            player_color: None,
        }
    }
}

impl PlayerStartResources {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        Ok(Self {
            gold: input.read_i32::<LE>()?,
            wood: input.read_i32::<LE>()?,
            food: input.read_i32::<LE>()?,
            stone: input.read_i32::<LE>()?,
            ore: if version >= 1.17 {
                input.read_i32::<LE>()?
            } else {
                100
            },
            goods: if version >= 1.17 {
                input.read_i32::<LE>()?
            } else {
                0
            },
            player_color: if version >= 1.24 {
                Some(input.read_i32::<LE>()?)
            } else {
                None
            },
        })
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        output.write_i32::<LE>(self.gold)?;
        output.write_i32::<LE>(self.wood)?;
        output.write_i32::<LE>(self.food)?;
        output.write_i32::<LE>(self.stone)?;
        if version >= 1.17 {
            output.write_i32::<LE>(self.ore)?;
            output.write_i32::<LE>(self.goods)?;
        }
        if version >= 1.24 {
            output.write_i32::<LE>(self.player_color.unwrap_or(0))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ScenarioPlayerData {
    pub name: Option<String>,
    pub view: (f32, f32),
    pub location: (i16, i16),
    pub allied_victory: bool,
    pub relations: Vec<i8>,
    pub unit_diplomacy: Vec<i32>,
    pub color: Option<i32>,
    pub victory: VictoryConditions,
}

impl ScenarioPlayerData {
    /// Get the default player name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|string| string.as_ref())
    }

    /// Set the default player name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Read player data from an input stream.
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let name = input.read_u16_length_prefixed_str()?;

        let view = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);

        let location = (input.read_i16::<LE>()?, input.read_i16::<LE>()?);

        let allied_victory = if version > 1.0 {
            input.read_u8()? != 0
        } else {
            false
        };

        let diplo_count = input.read_i16::<LE>()?;
        let mut relations = Vec::with_capacity(diplo_count as usize);
        for _ in 0..diplo_count {
            relations.push(input.read_i8()?);
        }

        let unit_diplomacy = if version >= 1.08 {
            vec![
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
            ]
        } else {
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0]
        };

        let color = if version >= 1.13 {
            Some(input.read_i32::<LE>()?)
        } else {
            None
        };

        let victory = VictoryConditions::read_from(&mut input, version >= 1.09)?;

        Ok(ScenarioPlayerData {
            name,
            view,
            location,
            allied_victory,
            relations,
            unit_diplomacy,
            color,
            victory,
        })
    }

    /// Write player data to an output stream.
    pub fn write_to(
        &self,
        mut output: impl Write,
        version: f32,
        victory_version: f32,
    ) -> Result<()> {
        write_opt_str(&mut output, &self.name)?;

        output.write_f32::<LE>(self.view.0)?;
        output.write_f32::<LE>(self.view.1)?;

        output.write_i16::<LE>(self.location.0)?;
        output.write_i16::<LE>(self.location.1)?;

        if version > 1.0 {
            output.write_u8(u8::from(self.allied_victory))?;
        };

        output.write_i16::<LE>(self.relations.len() as i16)?;
        for rel in &self.relations {
            output.write_i8(*rel)?;
        }

        if version >= 1.08 {
            output.write_i32::<LE>(self.unit_diplomacy[0])?;
            output.write_i32::<LE>(self.unit_diplomacy[1])?;
            output.write_i32::<LE>(self.unit_diplomacy[2])?;
            output.write_i32::<LE>(self.unit_diplomacy[3])?;
            output.write_i32::<LE>(self.unit_diplomacy[4])?;
            output.write_i32::<LE>(self.unit_diplomacy[5])?;
            output.write_i32::<LE>(self.unit_diplomacy[6])?;
            output.write_i32::<LE>(self.unit_diplomacy[7])?;
            output.write_i32::<LE>(self.unit_diplomacy[8])?;
        }

        if version >= 1.13 {
            output.write_i32::<LE>(self.color.unwrap_or(-1))?;
        }

        self.victory.write_to(
            &mut output,
            if version >= 1.09 {
                Some(victory_version)
            } else {
                None
            },
        )?;

        Ok(())
    }
}

/// Initial player attributes.
#[derive(Debug, Clone)]
pub struct WorldPlayerData {
    /// Initial food count.
    pub(crate) food: f32,
    /// Initial wood count.
    pub(crate) wood: f32,
    /// Initial gold count.
    pub(crate) gold: f32,
    /// Initial stone count.
    pub(crate) stone: f32,
    /// Initial ore count. (unused, except in some mods)
    pub(crate) ore: f32,
    /// Initial trade goods count. (unused)
    pub(crate) goods: f32,
    /// Max population.
    pub(crate) population: f32,
}

impl Default for WorldPlayerData {
    fn default() -> Self {
        Self {
            food: 200.0,
            wood: 200.0,
            gold: 100.0,
            stone: 200.0,
            ore: 100.0,
            goods: 0.0,
            population: 75.0,
        }
    }
}

impl WorldPlayerData {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        Ok(Self {
            food: if version > 1.06 {
                input.read_f32::<LE>()?
            } else {
                200.0
            },
            wood: if version > 1.06 {
                input.read_f32::<LE>()?
            } else {
                200.0
            },
            gold: if version > 1.06 {
                input.read_f32::<LE>()?
            } else {
                50.0
            },
            stone: if version > 1.06 {
                input.read_f32::<LE>()?
            } else {
                100.0
            },
            ore: if version > 1.12 {
                input.read_f32::<LE>()?
            } else {
                100.0
            },
            goods: if version > 1.12 {
                input.read_f32::<LE>()?
            } else {
                0.0
            },
            population: if version >= 1.14 {
                input.read_f32::<LE>()?
            } else {
                75.0
            },
        })
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        if version > 1.06 {
            output.write_f32::<LE>(self.food)?;
            output.write_f32::<LE>(self.wood)?;
            output.write_f32::<LE>(self.gold)?;
            output.write_f32::<LE>(self.stone)?;
        }
        if version > 1.12 {
            output.write_f32::<LE>(self.ore)?;
        }
        if version > 1.12 {
            output.write_f32::<LE>(self.goods)?;
        }
        if version >= 1.14 {
            output.write_f32::<LE>(self.population)?;
        }
        Ok(())
    }
}
