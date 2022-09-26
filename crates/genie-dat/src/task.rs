use crate::sound::SoundID;
use crate::sprite::SpriteID;
use crate::unit_type::UnitTypeID;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::read_opt_u16;
use std::convert::TryInto;
use std::io::{Read, Result, Write};
use std::ops::Deref;

#[derive(Debug, Default, Clone)]
pub struct TaskList(Vec<Task>);

#[derive(Debug, Default, Clone)]
pub struct Task {
    id: u16,
    is_default: bool,
    action_type: u16,
    object_class: i16,
    object_id: Option<UnitTypeID>,
    terrain_id: i16,
    attribute_types: (i16, i16, i16, i16),
    work_values: (f32, f32),
    work_range: f32,
    auto_search_targets: bool,
    search_wait_time: f32,
    enable_targeting: bool,
    combat_level: u8,
    work_flags: (u16, u16),
    owner_type: u8,
    holding_attribute: u8,
    state_building: u8,
    move_sprite: Option<SpriteID>,
    work_sprite: Option<SpriteID>,
    work_sprite2: Option<SpriteID>,
    carry_sprite: Option<SpriteID>,
    work_sound: Option<SoundID>,
    work_sound2: Option<SoundID>,
}

impl Deref for TaskList {
    type Target = Vec<Task>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TaskList {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let num_tasks = input.read_u16::<LE>()?;
        let mut tasks = vec![];
        for _ in 0..num_tasks {
            let task_type = input.read_u16::<LE>()?;
            assert_eq!(task_type, 1);
            tasks.push(Task::read_from(&mut input)?);
        }

        Ok(Self(tasks))
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u16::<LE>(self.len().try_into().unwrap())?;
        for task in self.iter() {
            output.write_u16::<LE>(1)?;
            task.write_to(output)?;
        }
        Ok(())
    }
}

impl Task {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        Ok(Task {
            id: input.read_u16::<LE>()?,
            is_default: input.read_u8()? != 0,
            action_type: input.read_u16::<LE>()?,
            object_class: input.read_i16::<LE>()?,
            object_id: read_opt_u16(&mut input)?,
            terrain_id: input.read_i16::<LE>()?,
            attribute_types: (
                input.read_i16::<LE>()?,
                input.read_i16::<LE>()?,
                input.read_i16::<LE>()?,
                input.read_i16::<LE>()?,
            ),
            work_values: (input.read_f32::<LE>()?, input.read_f32::<LE>()?),
            work_range: input.read_f32::<LE>()?,
            auto_search_targets: input.read_u8()? != 0,
            search_wait_time: input.read_f32::<LE>()?,
            enable_targeting: input.read_u8()? != 0,
            combat_level: input.read_u8()?,
            work_flags: (input.read_u16::<LE>()?, input.read_u16::<LE>()?),
            owner_type: input.read_u8()?,
            holding_attribute: input.read_u8()?,
            state_building: input.read_u8()?,
            move_sprite: read_opt_u16(&mut input)?,
            work_sprite: read_opt_u16(&mut input)?,
            work_sprite2: read_opt_u16(&mut input)?,
            carry_sprite: read_opt_u16(&mut input)?,
            work_sound: read_opt_u16(&mut input)?,
            work_sound2: read_opt_u16(&mut input)?,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u16::<LE>(self.id)?;
        output.write_u8(u8::from(self.is_default))?;
        output.write_u16::<LE>(self.action_type)?;
        output.write_i16::<LE>(self.object_class)?;
        output.write_i16::<LE>(
            self.object_id
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(self.terrain_id)?;
        output.write_i16::<LE>(self.attribute_types.0)?;
        output.write_i16::<LE>(self.attribute_types.1)?;
        output.write_i16::<LE>(self.attribute_types.2)?;
        output.write_i16::<LE>(self.attribute_types.3)?;
        output.write_f32::<LE>(self.work_values.0)?;
        output.write_f32::<LE>(self.work_values.1)?;
        output.write_f32::<LE>(self.work_range)?;
        output.write_u8(u8::from(self.auto_search_targets))?;
        output.write_f32::<LE>(self.search_wait_time)?;
        output.write_u8(u8::from(self.enable_targeting))?;
        output.write_u8(self.combat_level)?;
        output.write_u16::<LE>(self.work_flags.0)?;
        output.write_u16::<LE>(self.work_flags.1)?;
        output.write_u8(self.owner_type)?;
        output.write_u8(self.holding_attribute)?;
        output.write_u8(self.state_building)?;
        output.write_i16::<LE>(
            self.move_sprite
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.work_sprite
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.work_sprite2
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.carry_sprite
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.work_sound
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.work_sound2
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        Ok(())
    }
}
