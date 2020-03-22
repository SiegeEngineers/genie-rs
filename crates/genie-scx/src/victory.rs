use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use crate::types::VictoryCondition;
use std::io::{Read, Write};

/// AoE1's victory info.
///
/// This was replaced by VictoryConditions in AoE2.
#[derive(Debug, Clone, Default)]
pub struct LegacyVictoryInfo {
    pub object_type: i32,
    pub all_flag: bool,
    pub player_id: i32,
    pub dest_object_id: i32,
    pub area: (f32, f32, f32, f32),
    pub victory_type: i32,
    pub amount: i32,
    pub attribute: i32,
    pub object_id: i32,
    pub dest_object_id2: i32,
}

impl LegacyVictoryInfo {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let object_type = input.read_i32::<LE>()?;
        let all_flag = input.read_i32::<LE>()? != 0;
        let player_id = input.read_i32::<LE>()?;
        let dest_object_id = input.read_i32::<LE>()?;
        let area = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        let victory_type = input.read_i32::<LE>()?;
        let amount = input.read_i32::<LE>()?;
        let attribute = input.read_i32::<LE>()?;
        let object_id = input.read_i32::<LE>()?;
        let dest_object_id2 = input.read_i32::<LE>()?;
        // Should be 0 because they're pointers
        let _object = input.read_u32::<LE>()?;
        let _dest_object = input.read_u32::<LE>()?;

        Ok(Self {
            object_type,
            all_flag,
            player_id,
            dest_object_id,
            area,
            victory_type,
            amount,
            attribute,
            object_id,
            dest_object_id2,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.object_type)?;
        output.write_i32::<LE>(if self.all_flag { 1 } else { 0 })?;
        output.write_i32::<LE>(self.player_id)?;
        output.write_i32::<LE>(self.dest_object_id)?;
        output.write_f32::<LE>(self.area.0)?;
        output.write_f32::<LE>(self.area.1)?;
        output.write_f32::<LE>(self.area.2)?;
        output.write_f32::<LE>(self.area.3)?;
        output.write_i32::<LE>(self.victory_type)?;
        output.write_i32::<LE>(self.amount)?;
        output.write_i32::<LE>(self.attribute)?;
        output.write_i32::<LE>(self.object_id)?;
        output.write_i32::<LE>(self.dest_object_id2)?;
        output.write_u32::<LE>(0)?;
        output.write_u32::<LE>(0)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VictoryEntry {
    command: VictoryCondition,
    object_type: i32,
    player_id: i32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    number: i32,
    count: i32,
    source_object: i32,
    target_object: i32,
    victory_group: i8,
    ally_flag: i8,
    state: i8,
}

impl VictoryEntry {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let command = input.read_u8()?.into();
        let object_type = input.read_i32::<LE>()?;
        let player_id = input.read_i32::<LE>()?;
        let x0 = input.read_f32::<LE>()?;
        let y0 = input.read_f32::<LE>()?;
        let x1 = input.read_f32::<LE>()?;
        let y1 = input.read_f32::<LE>()?;
        let number = input.read_i32::<LE>()?;
        let count = input.read_i32::<LE>()?;
        let source_object = input.read_i32::<LE>()?;
        let target_object = input.read_i32::<LE>()?;
        let victory_group = input.read_i8()?;
        let ally_flag = input.read_i8()?;
        let state = input.read_i8()?;

        Ok(Self {
            command,
            object_type,
            player_id,
            x0,
            y0,
            x1,
            y1,
            number,
            count,
            source_object,
            target_object,
            victory_group,
            ally_flag,
            state,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.command.into())?;
        output.write_i32::<LE>(self.object_type)?;
        output.write_i32::<LE>(self.player_id)?;
        output.write_f32::<LE>(self.x0)?;
        output.write_f32::<LE>(self.y0)?;
        output.write_f32::<LE>(self.x1)?;
        output.write_f32::<LE>(self.y1)?;
        output.write_i32::<LE>(self.number)?;
        output.write_i32::<LE>(self.count)?;
        output.write_i32::<LE>(self.source_object)?;
        output.write_i32::<LE>(self.target_object)?;
        output.write_i8(self.victory_group)?;
        output.write_i8(self.ally_flag)?;
        output.write_i8(self.state)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VictoryPointEntry {
    command: i8,
    state: i8,
    attribute: i32,
    amount: i32,
    points: i32,
    current_points: i32,
    id: i8,
    group: i8,
    current_attribute_amount: f32,
    attribute1: i32,
    current_attribute_amount1: f32,
}

impl VictoryPointEntry {
    pub fn from<R: Read>(input: &mut R, version: f32) -> Result<Self> {
        let command = input.read_i8()?;
        let state = input.read_i8()?;
        let attribute = input.read_i32::<LE>()?;
        let amount = input.read_i32::<LE>()?;
        let points = input.read_i32::<LE>()?;
        let current_points = input.read_i32::<LE>()?;
        let id = input.read_i8()?;
        let group = input.read_i8()?;
        let current_attribute_amount = input.read_f32::<LE>()?;
        let (attribute1, current_attribute_amount1) = if version >= 2.0 {
            (input.read_i32::<LE>()?, input.read_f32::<LE>()?)
        } else {
            (-1, 0.0)
        };

        Ok(Self {
            command,
            state,
            attribute,
            amount,
            points,
            current_points,
            id,
            group,
            current_attribute_amount,
            attribute1,
            current_attribute_amount1,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: f32) -> Result<()> {
        output.write_i8(self.command)?;
        output.write_i8(self.state)?;
        output.write_i32::<LE>(self.attribute)?;
        output.write_i32::<LE>(self.amount)?;
        output.write_i32::<LE>(self.points)?;
        output.write_i32::<LE>(self.current_points)?;
        output.write_i8(self.id)?;
        output.write_i8(self.group)?;
        output.write_f32::<LE>(self.current_attribute_amount)?;
        if version >= 2.0 {
            output.write_i32::<LE>(self.attribute1)?;
            output.write_f32::<LE>(self.current_attribute_amount1)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VictoryConditions {
    version: f32,
    victory: bool,
    total_points: i32,
    starting_points: i32,
    starting_group: i32,
    entries: Vec<VictoryEntry>,
    point_entries: Vec<VictoryPointEntry>,
}

impl VictoryConditions {
    pub fn from<R: Read>(input: &mut R, has_version: bool) -> Result<Self> {
        let version = if has_version {
            input.read_f32::<LE>()?
        } else {
            0.0
        };

        let num_conditions = input.read_i32::<LE>()?;
        let victory = input.read_u8()? != 0;

        let mut entries = Vec::with_capacity(num_conditions as usize);
        for _ in 0..num_conditions {
            entries.push(VictoryEntry::from(input)?);
        }

        let mut total_points = 0;
        let mut point_entries = vec![];
        let mut starting_points = 0;
        let mut starting_group = 0;

        if version >= 1.0 {
            total_points = input.read_i32::<LE>()?;
            let num_point_entries = input.read_i32::<LE>()?;

            if version >= 2.0 {
                starting_points = input.read_i32::<LE>()?;
                starting_group = input.read_i32::<LE>()?;
            }

            for _ in 0..num_point_entries {
                point_entries.push(VictoryPointEntry::from(input, version)?);
            }
        }

        Ok(Self {
            version,
            victory,
            total_points,
            starting_points,
            starting_group,
            entries,
            point_entries,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: Option<f32>) -> Result<()> {
        if let Some(v) = version {
            output.write_f32::<LE>(v)?;
        }

        let version = version.unwrap_or(std::f32::MIN);

        output.write_i32::<LE>(self.entries.len() as i32)?;
        output.write_u8(if self.victory { 1 } else { 0 })?;

        for entry in &self.entries {
            entry.write_to(output)?;
        }

        if version >= 1.0 {
            output.write_i32::<LE>(self.total_points)?;
            output.write_i32::<LE>(self.point_entries.len() as i32)?;

            if version >= 2.0 {
                output.write_i32::<LE>(self.starting_points)?;
                output.write_i32::<LE>(self.starting_group)?;
            }

            for entry in &self.point_entries {
                entry.write_to(output, version)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct VictoryInfo {
    pub(crate) conquest: i32,
    pub(crate) ruins: i32,
    pub(crate) artifacts: i32,
    pub(crate) discoveries: i32,
    pub(crate) exploration: i32,
    pub(crate) gold: i32,
}

impl VictoryInfo {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            conquest: input.read_i32::<LE>()?,
            ruins: input.read_i32::<LE>()?,
            artifacts: input.read_i32::<LE>()?,
            discoveries: input.read_i32::<LE>()?,
            exploration: input.read_i32::<LE>()?,
            gold: input.read_i32::<LE>()?,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.conquest)?;
        output.write_i32::<LE>(self.ruins)?;
        output.write_i32::<LE>(self.artifacts)?;
        output.write_i32::<LE>(self.discoveries)?;
        output.write_i32::<LE>(self.exploration)?;
        output.write_i32::<LE>(self.gold)?;

        Ok(())
    }
}
