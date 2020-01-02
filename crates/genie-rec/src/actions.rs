use crate::{ObjectID, PlayerID, Result};
use genie_support::UnitTypeID;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Write};
use std::convert::TryInto;
use arrayvec::ArrayVec;

#[derive(Debug, Clone)]
pub struct ViewLock {
    pub x: f32,
    pub y: f32,
    pub player: PlayerID,
}

impl ViewLock {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let x = input.read_f32::<LE>()?;
        let y = input.read_f32::<LE>()?;
        let player = input.read_i32::<LE>()?.try_into().unwrap();
        Ok(Self { x, y, player })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_f32::<LE>(self.x)?;
        output.write_f32::<LE>(self.y)?;
        output.write_i32::<LE>(self.player.try_into().unwrap())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ObjectsList {
    /// Use the same objects as the previous command.
    SameAsLast,
    /// Use this list of objects.
    List(Vec<ObjectID>),
}

impl Default for ObjectsList {
    fn default() -> Self {
        ObjectsList::List(vec![])
    }
}

impl ObjectsList {
    pub fn read_from<R: Read>(input: &mut R, count: i32) -> Result<Self> {
        if count < 0xFF {
            let mut list = vec![];
            for _ in 0..count {
                list.push(input.read_i32::<LE>()?.try_into().unwrap());
            }
            Ok(ObjectsList::List(list))
        } else {
            Ok(ObjectsList::SameAsLast)
        }
    }
}

#[derive(Debug, Default)]
pub struct OrderCommand {
    player_id: PlayerID,
    target_id: ObjectID,
    x: f32,
    y: f32,
    objects: ObjectsList,
}

impl OrderCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        command.player_id = input.read_i8()?.try_into().unwrap();
        skip(input, 2)?;
        command.target_id = input.read_i32::<LE>()?.try_into().unwrap();
        let selected_count = input.read_i32::<LE>()?;
        command.x = input.read_f32::<LE>()?;
        command.y = input.read_f32::<LE>()?;
        command.objects = ObjectsList::read_from(input, selected_count)?;
        Ok(command)
    }
}

#[derive(Debug, Default)]
pub struct StopCommand {
    objects: ObjectsList,
}

impl StopCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = input.read_i8()?;
        command.objects = ObjectsList::read_from(input, selected_count as i32)?;
        Ok(command)
    }
}

#[derive(Debug, Default)]
pub struct WorkCommand {
    target_id: ObjectID,
    x: f32,
    y: f32,
    objects: ObjectsList,
}

impl WorkCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        skip(input, 3)?;
        command.target_id = input.read_i32::<LE>()?.try_into().unwrap();
        let selected_count = input.read_i8()?;
        skip(input, 3)?;
        command.x = input.read_f32::<LE>()?;
        command.y = input.read_f32::<LE>()?;
        command.objects = ObjectsList::read_from(input, selected_count as i32)?;
        Ok(command)
    }
}

#[derive(Debug, Clone)]
pub struct AddAttributeCommand {
    pub player_id: PlayerID,
    pub attribute: u8,
    pub amount: f32,
}

impl AddAttributeCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let player_id = input.read_u8()?.into();
        let attribute = input.read_u8()?;
        let _padding = input.read_u8()?;
        let amount = input.read_f32::<LE>()?;
        Ok(Self { player_id, attribute, amount })
    }
}

#[derive(Debug, Clone)]
pub struct GroupWaypointCommand {
    pub player_id: PlayerID,
    pub object_id: ObjectID,
    waypoints: i8,
}

impl GroupWaypointCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let player_id = input.read_u8()?.into();
        skip(input, 2)?;
        let object_id = input.read_u32::<LE>()?.into();
        let waypoints = input.read_i8()?;
        skip(input, 1)?;
        Ok(Self {
            player_id,
            object_id,
            waypoints,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UserPatchAICommand {
    player_id: PlayerID,
    ai_action: u8,
    params: ArrayVec<[u32; 4]>,
}

impl UserPatchAICommand {
    pub fn read_from<R: Read>(input: &mut R, size: u32) -> Result<Self> {
        let num_params = (size - 4) / 4;
        assert!(num_params < 4, "UserPatchAICommand needs more room for {} params", num_params);
        let ai_action = input.read_u8()?;
        let player_id = input.read_u16::<LE>()?.try_into().unwrap();
        let mut params: ArrayVec<[u32; 4]> = Default::default();
        for _ in 0..num_params {
            params.push(input.read_u32::<LE>()?);
        }
        Ok(Self {
            player_id,
            ai_action,
            params,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MakeCommand {
    pub player_id: PlayerID,
    pub building_id: ObjectID,
    pub unit_type_id: UnitTypeID,
    pub target_id: Option<ObjectID>,
}

impl MakeCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        skip(input, 3)?;
        let building_id = input.read_u32::<LE>()?.into();
        let player_id = input.read_u8()?.into();
        let _padding = input.read_u8()?;
        let unit_type_id = input.read_u16::<LE>()?.into();
        let target_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        Ok(Self {
            player_id,
            building_id,
            unit_type_id,
            target_id,
        })
    }
}

#[derive(Debug, Clone)]
pub enum GameCommand {
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

#[derive(Debug)]
struct RawGameCommand {
    game_command: u8,
    var1: i16,
    var2: i16,
    var3: f32,
    var4: u32,
}

impl RawGameCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let game_command = input.read_u8()?;
        let var1 = input.read_i16::<LE>()?;
        let var2 = input.read_i16::<LE>()?;
        let _padding = input.read_u16::<LE>()?;
        let var3 = input.read_f32::<LE>()?;
        let var4 = input.read_u32::<LE>()?;
        Ok(Self {
            game_command,
            var1,
            var2,
            var3,
            var4,
        })
    }
}

impl GameCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let RawGameCommand {
            game_command,
            var1,
            var2,
            var3,
            var4,
        } = RawGameCommand::read_from(input)?;

        use GameCommand::*;
        match game_command {
            0x01 => {
                Ok(SetGameSpeed {
                    player_id: var1.try_into().unwrap(),
                    speed: var3,
                })
            }
            0x0b => {
                Ok(SetStrategicNumber {
                    player_id: var1.try_into().unwrap(),
                    strategic_number: var2,
                    value: var4.try_into().unwrap(),
                })
            }
            _ => panic!("unimplemented game command {:#x}", game_command),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuildWallCommand {
    pub player_id: PlayerID,
    pub start: (u8, u8),
    pub end: (u8, u8),
    pub unit_type_id: UnitTypeID,
    pub builders: ObjectsList,
}

impl BuildWallCommand {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let selected_count = input.read_i8()?;
        let player_id = input.read_u8()?.into();
        let start = (input.read_u8()?,input.read_u8()?);
        let end = (input.read_u8()?,input.read_u8()?);
        let _padding = input.read_u8()?;
        let unit_type_id = input.read_u16::<LE>()?.into();
        let _padding = input.read_u16::<LE>()?;
        assert_eq!(input.read_u32::<LE>()?, 0xFFFF_FFFF, "check out what this is for");
        let builders = if selected_count == -1 {
            ObjectsList::SameAsLast
        } else {
            let mut list = vec![0; selected_count.try_into().unwrap()];
            for object in list.iter_mut() {
                *object = input.read_i32::<LE>()?;
            }
            if selected_count == 1 && list[0] == -1 {
                list.clear();
            }
            ObjectsList::List(list.into_iter().map(|id| id.try_into().unwrap()).collect())
        };
        Ok(Self {
            player_id,
            start,
            end,
            unit_type_id,
            builders,
        })
    }
}

#[derive(Debug)]
pub enum Command {
    Order(OrderCommand),
    Stop(StopCommand),
    Work(WorkCommand),
    AddAttribute(AddAttributeCommand),
    GroupWaypoint(GroupWaypointCommand),
    UserPatchAI(UserPatchAICommand),
    Make(MakeCommand),
    Game(GameCommand),
    BuildWall(BuildWallCommand),
}

impl Command {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let len = input.read_u32::<LE>()?;
        let command = match input.read_u8()? {
            0x00 => OrderCommand::read_from(input).map(Command::Order),
            0x01 => StopCommand::read_from(input).map(Command::Stop),
            0x02 => WorkCommand::read_from(input).map(Command::Work),
            0x05 => AddAttributeCommand::read_from(input).map(Command::AddAttribute),
            0x10 => GroupWaypointCommand::read_from(input).map(Command::GroupWaypoint),
            0x35 => UserPatchAICommand::read_from(input, len).map(Command::UserPatchAI),
            0x64 => MakeCommand::read_from(input).map(Command::Make),
            0x67 => GameCommand::read_from(input).map(Command::Game),
            0x69 => BuildWallCommand::read_from(input).map(Command::BuildWall),
            id => panic!("unsupported command type {:#x}", id),
        };
        let _world_time = input.read_u32::<LE>()?;
        command
    }
}

#[derive(Debug, Default)]
pub struct Sync {
    sequence: Option<u8>,
    time: i32,
    checksums: Option<(u32, u32, u32)>,
}

impl Sync {
    pub fn read_from<R: Read>(input: &mut R, use_sequence_numbers: bool, includes_checksum: bool) -> Result<Self> {
        let mut sync = Self::default();
        sync.sequence = if use_sequence_numbers { Some(input.read_u8()?) } else { None };
        sync.time = input.read_i32::<LE>()?;
        sync.checksums = if includes_checksum {
            let _always_zero = input.read_u32::<LE>()?;
            let _always_zero = input.read_u32::<LE>()?;
            let checksum = input.read_u32::<LE>()?;
            let _always_zero = input.read_u32::<LE>()?;
            let position_checksum = input.read_u32::<LE>()?;
            let _always_zero = input.read_u32::<LE>()?;
            let _always_zero = input.read_u32::<LE>()?;
            let _always_zero = input.read_u32::<LE>()?;
            let action_checksum = input.read_u32::<LE>()?;
            let _always_zero = input.read_u32::<LE>()?;
            Some((checksum, position_checksum, action_checksum))
        } else { None };
        Ok(sync)
    }
}

#[derive(Debug)]
pub struct Meta {
    pub checksum_interval: u32,
    pub is_multiplayer: bool,
    pub use_sequence_numbers: bool,
    pub local_player_id: PlayerID,
    pub header_position: u32,
    pub num_chapters: u32,
}

impl Meta {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let checksum_interval = input.read_u32::<LE>()?;
        let is_multiplayer = input.read_u32::<LE>()? != 0;
        let local_player_id = input.read_u32::<LE>()?.try_into().unwrap();
        let header_position = input.read_u32::<LE>()?;
        let use_sequence_numbers = input.read_u32::<LE>()? != 0;
        let num_chapters = input.read_u32::<LE>()?;
        Ok(Self {
            checksum_interval,
            is_multiplayer,
            use_sequence_numbers,
            local_player_id,
            header_position,
            num_chapters,
        })
    }
}

#[derive(Debug)]
pub enum Action {
    Command(Command),
    Sync(Sync),
    ViewLock(ViewLock),
    Meta(Meta),
}

fn skip<R: Read>(input: &mut R, bytes: u64) -> Result<()> {
    std::io::copy(&mut input.by_ref().take(bytes), &mut std::io::sink())?;
    Ok(())
}
