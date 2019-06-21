use crate::{ObjectID, PlayerID};
use auto_serializer::auto_serialize;
use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Result};

#[derive(Debug)]
pub struct ViewLock {
    pub x: f32,
    pub y: f32,
    pub player: i32,
}

auto_serialize!(ViewLock, {
    x: F32LE,
    y: F32LE,
    player: I32LE,
});

#[derive(Debug)]
pub enum ObjectsList {
    SameAsLast,
    List(Vec<ObjectID>),
}

impl Default for ObjectsList {
    fn default() -> Self {
        ObjectsList::List(vec![])
    }
}

impl ObjectsList {
    pub fn read<R: Read>(input: &mut R, count: i32) -> Result<Self> {
        if count < 0xFF {
            let mut list = vec![];
            for _ in 0..count {
                list.push(input.read_i32::<LE>()?);
            }
            Ok(ObjectsList::List(list))
        } else {
            Ok(ObjectsList::SameAsLast)
        }
    }
}

#[derive(Debug, Default)]
pub struct OrderAction {
    player_id: PlayerID,
    target_id: ObjectID,
    x: f32,
    y: f32,
    objects: ObjectsList,
}

impl OrderAction {
    pub fn read<R: Read>(input: &mut R) -> Result<Self> {
        let mut action = Self::default();
        action.player_id = input.read_i8()?;
        skip(input, 2)?;
        action.target_id = input.read_i32::<LE>()?;
        let selected_count = input.read_i32::<LE>()?;
        action.x = input.read_f32::<LE>()?;
        action.y = input.read_f32::<LE>()?;
        action.objects = ObjectsList::read(input, selected_count)?;
        Ok(action)
    }
}

#[derive(Debug, Default)]
pub struct StopAction {
    objects: ObjectsList,
}

impl StopAction {
    pub fn read<R: Read>(input: &mut R) -> Result<Self> {
        let mut action = Self::default();
        let selected_count = input.read_i8()?;
        action.objects = ObjectsList::read(input, selected_count as i32)?;
        Ok(action)
    }
}

#[derive(Debug, Default)]
pub struct WorkAction {
    target_id: ObjectID,
    x: f32,
    y: f32,
    objects: ObjectsList,
}

impl WorkAction {
    pub fn read<R: Read>(input: &mut R) -> Result<Self> {
        let mut action = Self::default();
        skip(input, 3)?;
        action.target_id = input.read_i32::<LE>()?;
        let selected_count = input.read_i8()?;
        skip(input, 3)?;
        action.x = input.read_f32::<LE>()?;
        action.y = input.read_f32::<LE>()?;
        action.objects = ObjectsList::read(input, selected_count as i32)?;
        Ok(action)
    }
}

pub enum GameSubAction {}

#[derive(Debug)]
pub enum GameAction {
    SetGameSpeed {
        player_id: PlayerID,
        speed: f32,
    },
    SetStrategicNumber {
        player_id: PlayerID,
        strategic_number: i16,
        value: i32,
    },
}

impl GameAction {
    pub fn read<R: Read>(input: &mut R) -> Result<Self> {
        let command = input.read_i8()?;
        let player_id = input.read_i8()?;

        use GameAction::*;
        match command {
            0x01 => {
                skip(input, 5)?;
                let speed = input.read_f32::<LE>()?;
                skip(input, 4)?;
                Ok(SetGameSpeed { player_id, speed })
            }
            0x0b => {
                let _padding = input.read_i8()?;
                let strategic_number = input.read_i16::<LE>()?;
                let value = input.read_i32::<LE>()?;
                skip(input, 6)?;
                Ok(SetStrategicNumber {
                    player_id,
                    strategic_number,
                    value,
                })
            }
            _ => panic!("unimplemented game action {:#x}", command),
        }
    }
}

#[derive(Debug)]
pub enum Action {
    Order(OrderAction),
    Stop(StopAction),
    Work(WorkAction),
    Game(GameAction),
}

impl Action {
    pub fn read<R: Read>(input: &mut R) -> Result<Self> {
        match input.read_u8()? {
            0x00 => OrderAction::read(input).map(Action::Order),
            0x01 => StopAction::read(input).map(Action::Stop),
            0x02 => WorkAction::read(input).map(Action::Work),
            0x67 => GameAction::read(input).map(Action::Game),
            id => panic!("unsupported action type {:#x}", id),
        }
    }
}

fn skip<R: Read>(input: &mut R, bytes: u64) -> Result<u64> {
    std::io::copy(&mut input.by_ref().take(bytes), &mut std::io::sink())
}
