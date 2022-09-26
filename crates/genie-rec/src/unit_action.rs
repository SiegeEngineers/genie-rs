use crate::element::{ReadableHeaderElement, WritableHeaderElement};
use crate::reader::RecordingHeaderReader;
use crate::ObjectID;
use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
pub use genie_dat::sprite::SpriteID;
pub use genie_support::UnitTypeID;
use genie_support::{read_opt_u16, read_opt_u32};
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct UnitAction {
    pub state: u32,
    pub target_object_id: Option<ObjectID>,
    pub target_object_id_2: Option<ObjectID>,
    pub target_position: (f32, f32, f32),
    pub timer: f32,
    pub target_moved_state: u8,
    pub task_id: Option<u16>,
    pub sub_action_value: u8,
    pub sub_actions: Vec<UnitAction>,
    pub sprite_id: Option<SpriteID>,
    pub params: ActionType,
}

impl ReadableHeaderElement for UnitAction {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let action_type = input.read_u16::<LE>()?;
        Self::read_from_inner(input, action_type)
    }
}

impl UnitAction {
    fn read_from_inner<R: Read>(
        input: &mut RecordingHeaderReader<R>,
        action_type: u16,
    ) -> Result<Self> {
        // TODO this is different between AoC 1.0 and AoC 1.0c. This version check is a guess
        // and may not actually be when it changed. May have to become more specific in the
        // future!
        let state = if input.version() <= 11.76 {
            input.read_u8()? as u32
        } else {
            input.read_u32::<LE>()?
        };
        let _target_object_pointer = input.read_u32::<LE>()?;
        let _target_object_pointer_2 = input.read_u32::<LE>()?;
        let target_object_id = read_opt_u32(input)?;
        let target_object_id_2 = read_opt_u32(input)?;
        let target_position = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        let timer = input.read_f32::<LE>()?;
        let target_moved_state = input.read_u8()?;
        let task_id = read_opt_u16(input)?;
        let sub_action_value = input.read_u8()?;
        let sub_actions = UnitAction::read_list_from(input)?;
        let sprite_id = read_opt_u16(input)?;
        let params = ActionType::read_from(input, action_type)?;

        Ok(UnitAction {
            state,
            target_object_id,
            target_object_id_2,
            target_position,
            timer,
            target_moved_state,
            task_id,
            sub_action_value,
            sub_actions,
            sprite_id,
            params,
        })
    }

    pub fn read_list_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Vec<Self>> {
        let mut list = vec![];
        loop {
            let action_type = input.read_u16::<LE>()?;
            if action_type == 0 {
                return Ok(list);
            }
            let action = Self::read_from_inner(input, action_type)?;
            list.push(action);
        }
    }
}

#[derive(Debug, Clone)]
pub enum ActionType {
    MoveTo(ActionMoveTo),
    Enter(ActionEnter),
    Explore,
    Attack(ActionAttack),
    Bird,
    Transport,
    Guard,
    Make(ActionMake),
    Artifact,
    Unknown(u16),
}

impl ActionType {
    pub fn read_from<R: Read>(
        input: &mut RecordingHeaderReader<R>,
        action_type: u16,
    ) -> Result<Self> {
        println!("{} {}", input.position(), action_type);

        let data = match action_type {
            1 => Self::MoveTo(ActionMoveTo::read_from(input)?),
            3 => Self::Enter(ActionEnter::read_from(input)?),
            4 => Self::Explore,
            9 => Self::Attack(ActionAttack::read_from(input)?),
            10 => Self::Bird,
            12 => Self::Transport,
            13 => Self::Guard,
            21 => Self::Make(ActionMake::read_from(input)?),
            107 => Self::Artifact,
            _ => unimplemented!("action type {} not yet implemented", action_type),
        };
        Ok(data)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ActionMoveTo {
    pub range: f32,
}

impl ReadableHeaderElement for ActionMoveTo {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let range = input.read_f32::<LE>()?;
        Ok(Self { range })
    }
}

impl WritableHeaderElement for ActionMoveTo {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_f32::<LE>(self.range)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ActionEnter {
    pub first_time: u32,
}

impl ReadableHeaderElement for ActionEnter {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let first_time = input.read_u32::<LE>()?;
        Ok(Self { first_time })
    }
}

impl WritableHeaderElement for ActionEnter {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u32::<LE>(self.first_time)?;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct ActionAttack {
    range: f32,
    min_range: f32,
    missile_id: UnitTypeID,
    frame_delay: u16,
    need_to_attack: u16,
    was_same_owner: u16,
    indirect_fire_flag: u8,
    move_sprite_id: Option<SpriteID>,
    fight_sprite_id: Option<SpriteID>,
    wait_sprite_id: Option<SpriteID>,
    last_target_position: (f32, f32, f32),
}

impl ReadableHeaderElement for ActionAttack {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        Ok(ActionAttack {
            range: input.read_f32::<LE>()?,
            min_range: input.read_f32::<LE>()?,
            missile_id: input.read_u16::<LE>()?.into(),
            frame_delay: input.read_u16::<LE>()?,
            need_to_attack: input.read_u16::<LE>()?,
            was_same_owner: input.read_u16::<LE>()?,
            indirect_fire_flag: input.read_u8()?,
            move_sprite_id: read_opt_u16(input)?,
            fight_sprite_id: read_opt_u16(input)?,
            wait_sprite_id: read_opt_u16(input)?,
            last_target_position: (
                input.read_f32::<LE>()?,
                input.read_f32::<LE>()?,
                input.read_f32::<LE>()?,
            ),
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ActionMake {
    pub work_timer: f32,
}

impl ReadableHeaderElement for ActionMake {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let work_timer = input.read_f32::<LE>()?;
        Ok(Self { work_timer })
    }
}

impl WritableHeaderElement for ActionMake {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_f32::<LE>(self.work_timer)?;
        Ok(())
    }
}
