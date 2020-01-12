use crate::Result;
use crate::{ObjectID, PlayerID};
use std::convert::{TryFrom, TryInto};
use std::io::{Read, Write};
use std::cmp;
use std::fmt;
use arrayvec::ArrayVec;
pub use genie_support::UnitTypeID;
pub use genie_dat::SpriteID;
use byteorder::{LE, ReadBytesExt, WriteBytesExt};

#[derive(Debug, Clone)]
pub struct UnitAction {
    pub state: u8,
    pub target_object_id: Option<ObjectID>,
    pub target_object_id_2: Option<ObjectID>,
    pub target_position: (f32, f32, f32),
    pub timer: f32,
    pub target_moved_state: u8,
    pub task_id: u16,
    pub sub_action_value: u8,
    pub sub_actions: Vec<UnitAction>,
    pub params: ActionType,
    pub sprite_id: Option<SpriteID>,
}

impl UnitAction {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let action_type = input.read_u16::<LE>()?;
        Self::read_from_inner(&mut input, action_type)
    }

    // `dyn` because this is a recursive function; taking &mut from a `impl Read` here
    // would cause infinite recursion in the types.
    fn read_from_inner(mut input: &mut dyn Read, action_type: u16) -> Result<Self> {
        let state = input.read_u8()?;
        let _padding = input.read_u8()?;
        let _padding = input.read_u8()?;
        let _padding = input.read_u8()?;
        let _target_object_pointer = input.read_u32::<LE>()?;
        let _target_object_pointer_2 = input.read_u32::<LE>()?;
        let target_object_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        let target_object_id_2 = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        let target_position = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        let timer = input.read_f32::<LE>()?;
        let target_moved_state = input.read_u8()?;
        let task_id = input.read_u16::<LE>()?;
        let sub_action_value = input.read_u8()?;
        let sub_actions = UnitAction::read_list_from(&mut input)?;
        let sprite_id = match input.read_i16::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        let params = ActionType::read_from(&mut input, action_type)?;

        Ok(Self {
            state,
            target_object_id,
            target_object_id_2,
            target_position,
            timer,
            target_moved_state,
            task_id,
            sub_action_value,
            sub_actions,
            params,
            sprite_id,
        })
    }

    pub fn read_list_from(mut input: impl Read) -> Result<Vec<Self>> {
        let mut list = vec![];
        loop {
            let action_type = input.read_u16::<LE>()?;
            if action_type == 0 {
                return Ok(list);
            }
            let action = Self::read_from_inner(&mut input, action_type)?;
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
}

impl ActionType {
    pub fn read_from(mut input: impl Read, action_type: u16) -> Result<Self> {
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

#[derive(Debug, Default ,Clone)]
pub struct ActionMoveTo {
    pub range: f32,
}

impl ActionMoveTo {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let range = input.read_f32::<LE>()?;
        Ok(Self { range })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_f32::<LE>(self.range)?;
        Ok(())
    }
}

#[derive(Debug, Default ,Clone)]
pub struct ActionEnter {
    pub first_time: u32,
}

impl ActionEnter {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let first_time = input.read_u32::<LE>()?;
        Ok(Self { first_time })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_u32::<LE>(self.first_time)?;
        Ok(())
    }
}

#[derive(Debug, Default ,Clone)]
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

impl ActionAttack {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut props = Self::default();
        props.range = input.read_f32::<LE>()?;
        props.min_range = input.read_f32::<LE>()?;
        props.missile_id = input.read_u16::<LE>()?.into();
        props.frame_delay = input.read_u16::<LE>()?;
        props.need_to_attack = input.read_u16::<LE>()?;
        props.was_same_owner = input.read_u16::<LE>()?;
        props.indirect_fire_flag = input.read_u8()?;
        props.move_sprite_id = match input.read_i16::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        props.fight_sprite_id = match input.read_i16::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        props.wait_sprite_id = match input.read_i16::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        props.last_target_position = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        Ok(props)
    }
}

#[derive(Debug, Default ,Clone)]
pub struct ActionMake {
    pub work_timer: f32,
}

impl ActionMake {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let work_timer = input.read_f32::<LE>()?;
        Ok(Self { work_timer })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_f32::<LE>(self.work_timer)?;
        Ok(())
    }
}

