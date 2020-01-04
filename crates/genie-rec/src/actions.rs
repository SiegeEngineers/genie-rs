use crate::{ObjectID, PlayerID, Result};
use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{TechID, UnitTypeID};
use std::convert::TryInto;
use std::io::{Read, Write};

#[derive(Debug, Default, Clone)]
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

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        if let ObjectsList::List(list) = self {
            for entry in list.iter().cloned() {
                output.write_u32::<LE>(entry.into())?;
            }
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        match self {
            ObjectsList::SameAsLast => 0,
            ObjectsList::List(list) => list.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Default, Clone)]
pub struct OrderCommand {
    pub player_id: PlayerID,
    pub target_id: ObjectID,
    pub x: f32,
    pub y: f32,
    pub objects: ObjectsList,
}

impl OrderCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        command.player_id = input.read_u8()?.into();
        skip(input, 2)?;
        command.target_id = input.read_i32::<LE>()?.try_into().unwrap();
        let selected_count = input.read_i32::<LE>()?;
        command.x = input.read_f32::<LE>()?;
        command.y = input.read_f32::<LE>()?;
        command.objects = ObjectsList::read_from(input, selected_count)?;
        Ok(command)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.player_id.into())?;
        output.write_all(&[0, 0])?;
        output.write_u32::<LE>(self.target_id.into())?;
        output.write_u32::<LE>(self.objects.len().try_into().unwrap())?;
        output.write_f32::<LE>(self.x)?;
        output.write_f32::<LE>(self.y)?;
        self.objects.write_to(output)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct StopCommand {
    pub objects: ObjectsList,
}

impl StopCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = input.read_i8()?;
        command.objects = ObjectsList::read_from(input, selected_count as i32)?;
        Ok(command)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i8(self.objects.len().try_into().unwrap())?;
        self.objects.write_to(output)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct WorkCommand {
    pub target_id: Option<ObjectID>,
    pub x: f32,
    pub y: f32,
    pub objects: ObjectsList,
}

impl WorkCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        skip(input, 3)?;
        command.target_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        let selected_count = input.read_i8()?;
        skip(input, 3)?;
        command.x = input.read_f32::<LE>()?;
        command.y = input.read_f32::<LE>()?;
        command.objects = ObjectsList::read_from(input, selected_count as i32)?;
        Ok(command)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_all(&[0, 0, 0])?;
        output.write_i32::<LE>(self.target_id.map(|u| u32::from(u) as i32).unwrap_or(-1))?;
        output.write_i8(self.objects.len().try_into().unwrap())?;
        output.write_all(&[0, 0, 0])?;
        output.write_f32::<LE>(self.x)?;
        output.write_f32::<LE>(self.y)?;
        self.objects.write_to(output)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct CreateCommand {
    pub unit_type_id: UnitTypeID,
    pub player_id: PlayerID,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl CreateCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        let _padding = input.read_u8()?;
        command.unit_type_id = input.read_u16::<LE>()?.into();
        command.player_id = input.read_u8()?.into();
        let _padding = input.read_u8()?;
        command.x = input.read_f32::<LE>()?;
        command.y = input.read_f32::<LE>()?;
        command.z = input.read_f32::<LE>()?;
        Ok(command)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(0)?;
        output.write_u16::<LE>(self.unit_type_id.into())?;
        output.write_u8(self.player_id.into())?;
        output.write_u8(0)?;
        output.write_f32::<LE>(self.x)?;
        output.write_f32::<LE>(self.y)?;
        output.write_f32::<LE>(self.z)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
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
        Ok(Self {
            player_id,
            attribute,
            amount,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.player_id.into())?;
        output.write_u8(self.attribute)?;
        output.write_u8(0)?;
        output.write_f32::<LE>(self.amount)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIOrderCommand {
    pub player_id: PlayerID,
    pub issuer: PlayerID,
    pub objects: ObjectsList,
    pub order_type: u16,
    pub order_priority: i8,
    pub target_id: Option<ObjectID>,
    pub target_player_id: Option<PlayerID>,
    pub target_location: (f32, f32, f32),
    pub range: f32,
    pub immediate: bool,
    pub add_to_front: bool,
}

impl AIOrderCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = i32::from(input.read_i8()?);
        command.player_id = input.read_u8()?.into();
        command.issuer = input.read_u8()?.into();
        let object_id = input.read_u32::<LE>()?;
        command.order_type = input.read_u16::<LE>()?;
        command.order_priority = input.read_i8()?;
        let _padding = input.read_u8()?;
        command.target_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        command.target_player_id = match input.read_i8()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        skip(input, 3)?;
        command.target_location = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        command.range = input.read_f32::<LE>()?;
        command.immediate = input.read_u8()? != 0;
        command.add_to_front = input.read_u8()? != 0;
        let _padding = input.read_u16::<LE>()?;
        command.objects = if selected_count == 1 {
            ObjectsList::List(vec![object_id.into()])
        } else {
            ObjectsList::read_from(input, selected_count)?
        };
        Ok(command)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i8(self.objects.len().try_into().unwrap())?;
        output.write_u8(self.player_id.into())?;
        output.write_u8(self.issuer.into())?;
        match &self.objects {
            ObjectsList::List(list) if list.len() == 1 => {
                output.write_u32::<LE>(list[0].into())?;
            }
            _ => output.write_i32::<LE>(-1)?,
        }
        output.write_u16::<LE>(self.order_type)?;
        output.write_i8(self.order_priority)?;
        output.write_u8(0)?;
        output.write_i32::<LE>(match self.target_id {
            Some(id) => id.try_into().unwrap(),
            None => -1,
        })?;
        output.write_u8(self.player_id.into())?;
        output.write_all(&[0, 0, 0])?;
        output.write_f32::<LE>(self.target_location.0)?;
        output.write_f32::<LE>(self.target_location.1)?;
        output.write_f32::<LE>(self.target_location.2)?;
        output.write_f32::<LE>(self.range)?;
        output.write_u8(if self.immediate { 1 } else { 0 })?;
        output.write_u8(if self.add_to_front { 1 } else { 0 })?;
        output.write_all(&[0, 0])?;
        if self.objects.len() > 1 {
            self.objects.write_to(output)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
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

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.player_id.into())?;
        output.write_all(&[0, 0])?;
        output.write_u32::<LE>(self.object_id.into())?;
        output.write_i8(self.waypoints)?;
        output.write_u8(0)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct UnitAIStateCommand {
    pub state: i8,
    pub objects: ObjectsList,
}

impl UnitAIStateCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let selected_count = input.read_u8()?;
        let state = input.read_i8()?;
        let objects = ObjectsList::read_from(input, i32::from(selected_count))?;
        Ok(Self { state, objects })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.objects.len().try_into().unwrap())?;
        output.write_i8(self.state)?;
        self.objects.write_to(output)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct UserPatchAICommand {
    pub player_id: PlayerID,
    /// 0: move to object
    /// 1: set unit ai state
    /// 2: ?
    /// 3: ?
    /// 4: stop unit group?
    /// 5: dropoff something?
    /// 6: dropoff something?
    /// 7: ?
    /// 8: set offensive target priority
    /// 9: reset offensive target priorities?
    /// 10: nothing?
    /// 11: stop unit group?
    /// 12: set gather point to garrison in self
    /// 13: set ai player name
    /// 14: unload object
    /// 15: nothing?
    pub ai_action: u8,
    pub params: ArrayVec<[u32; 4]>,
}

impl UserPatchAICommand {
    pub fn read_from<R: Read>(input: &mut R, size: u32) -> Result<Self> {
        let num_params = (size - 4) / 4;
        assert!(
            num_params < 4,
            "UserPatchAICommand needs more room for {} params",
            num_params
        );
        let ai_action = input.read_u8()?;
        let player_id = input.read_u8()?.into();
        let _padding = input.read_u8()?;
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

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.ai_action)?;
        output.write_u8(self.player_id.into())?;
        output.write_u8(0)?;
        for p in &self.params {
            output.write_u32::<LE>(*p)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
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

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_all(&[0, 0, 0])?;
        output.write_u32::<LE>(self.building_id.into())?;
        output.write_u8(self.player_id.into())?;
        output.write_u8(0)?;
        output.write_u16::<LE>(self.unit_type_id.into())?;
        output.write_i32::<LE>(match self.target_id {
            None => -1,
            Some(id) => id.try_into().unwrap(),
        })?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ResearchCommand {
    pub player_id: PlayerID,
    pub building_id: ObjectID,
    pub tech_id: TechID,
    pub target_id: Option<ObjectID>,
}

impl ResearchCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        skip(input, 3)?;
        let building_id = input.read_u32::<LE>()?.into();
        let player_id = input.read_u8()?.into();
        let _padding = input.read_u8()?;
        let tech_id = input.read_u16::<LE>()?.into();
        let target_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        Ok(Self {
            player_id,
            building_id,
            tech_id,
            target_id,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_all(&[0, 0, 0])?;
        output.write_u32::<LE>(self.building_id.into())?;
        output.write_u8(self.player_id.into())?;
        output.write_u8(0)?;
        output.write_u16::<LE>(self.tech_id.into())?;
        output.write_i32::<LE>(match self.target_id {
            None => -1,
            Some(id) => id.try_into().unwrap(),
        })?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct BuildCommand {
    /// The ID of the player issuing this command.
    pub player_id: PlayerID,
    /// The type of building to place.
    pub unit_type_id: UnitTypeID,
    /// The location of the new building foundation.
    pub location: (f32, f32),
    /// The index of the frame to use, for buildings with multiple graphics like houses.
    pub frame: u8,
    /// The IDs of the villagers that are tasked to build this building.
    pub builders: ObjectsList,
    /// Unique ID for the _command_ (not building)? Used by AIs?
    unique_id: Option<u32>,
}

impl BuildCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = input.read_i8()?;
        command.player_id = input.read_u8()?.into();
        let _padding = input.read_u8()?;
        let x = input.read_f32::<LE>()?;
        let y = input.read_f32::<LE>()?;
        command.location = (x, y);
        command.unit_type_id = input.read_u16::<LE>()?.into();
        let _padding = input.read_u16::<LE>()?;
        command.unique_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        command.frame = input.read_u8()?;
        skip(input, 3)?;
        command.builders = ObjectsList::read_from(input, i32::from(selected_count))?;
        Ok(command)
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
            0x01 => Ok(SetGameSpeed {
                player_id: var1.try_into().unwrap(),
                speed: var3,
            }),
            0x0b => Ok(SetStrategicNumber {
                player_id: var1.try_into().unwrap(),
                strategic_number: var2,
                value: var4.try_into().unwrap(),
            }),
            _ => panic!("unimplemented game command {:#x}", game_command),
        }
    }
}

#[derive(Debug, Default, Clone)]
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
        let start = (input.read_u8()?, input.read_u8()?);
        let end = (input.read_u8()?, input.read_u8()?);
        let _padding = input.read_u8()?;
        let unit_type_id = input.read_u16::<LE>()?.into();
        let _padding = input.read_u16::<LE>()?;
        assert_eq!(
            input.read_u32::<LE>()?,
            0xFFFF_FFFF,
            "check out what this is for"
        );
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

#[derive(Debug, Default, Clone)]
pub struct UngarrisonCommand {
    pub ungarrison_type: i8,
    pub unit_type_id: Option<ObjectID>,
    pub location: Option<(f32, f32)>,
    pub objects: ObjectsList,
}

impl UngarrisonCommand {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = input.read_i8()?;
        let _padding = input.read_u16::<LE>()?;
        let x = input.read_f32::<LE>()?;
        let y = input.read_f32::<LE>()?;
        command.location = if x != -1.0 && y != -1.0 {
            Some((x, y))
        } else {
            None
        };
        command.ungarrison_type = input.read_i8()?;
        skip(input, 3)?;
        command.unit_type_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        command.objects = ObjectsList::read_from(input, i32::from(selected_count))?;
        Ok(command)
    }
}

#[derive(Debug, Default, Clone)]
pub struct FlareCommand {
    pub player_id: PlayerID,
    pub comm_player_id: PlayerID,
    pub recipients: [bool; 9],
    pub location: (f32, f32),
}

impl FlareCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        skip(input, 3)?;
        assert_eq!(
            input.read_i32::<LE>()?,
            -1,
            "found flare with unexpected unit id"
        );
        for receive in command.recipients.iter_mut() {
            *receive = input.read_u8()? != 0;
        }
        skip(input, 3)?;
        command.location = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        command.player_id = input.read_u8()?.into();
        command.comm_player_id = input.read_u8()?.into();
        skip(input, 2)?;
        Ok(command)
    }
}

#[derive(Debug, Default, Clone)]
pub struct UnitOrderCommand {
    pub target_id: Option<ObjectID>,
    pub action: i8,
    pub param: Option<u8>,
    pub location: Option<(f32, f32)>,
    pub unique_id: Option<u32>,
    pub objects: ObjectsList,
}

impl UnitOrderCommand {
    fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = input.read_i8()?;
        let _padding = input.read_u16::<LE>()?;
        command.target_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        command.action = input.read_i8()?;
        command.param = match input.read_i8()? {
            -1 => None,
            param => Some(param as u8),
        };
        let _padding = input.read_u16::<LE>()?;
        let x = input.read_f32::<LE>()?;
        let y = input.read_f32::<LE>()?;
        command.location = if x != -1.0 && y != -1.0 {
            Some((x, y))
        } else {
            None
        };
        command.unique_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id as u32),
        };
        command.objects = ObjectsList::read_from(input, i32::from(selected_count))?;
        Ok(command)
    }
}

#[derive(Debug, Default, Clone)]
pub struct BackToWorkCommand {
    pub building_id: ObjectID,
}

impl BackToWorkCommand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        skip(input, 3)?;
        let building_id = input.read_u32::<LE>()?.into();
        Ok(Self { building_id })
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Order(OrderCommand),
    Stop(StopCommand),
    Work(WorkCommand),
    Create(CreateCommand),
    AddAttribute(AddAttributeCommand),
    AIOrder(AIOrderCommand),
    GroupWaypoint(GroupWaypointCommand),
    UnitAIState(UnitAIStateCommand),
    UserPatchAI(UserPatchAICommand),
    Make(MakeCommand),
    Research(ResearchCommand),
    Build(BuildCommand),
    Game(GameCommand),
    BuildWall(BuildWallCommand),
    Ungarrison(UngarrisonCommand),
    Flare(FlareCommand),
    UnitOrder(UnitOrderCommand),
    BackToWork(BackToWorkCommand),
}

impl Command {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let len = input.read_u32::<LE>()?;
        let command = match input.read_u8()? {
            0x00 => OrderCommand::read_from(input).map(Command::Order),
            0x01 => StopCommand::read_from(input).map(Command::Stop),
            0x02 => WorkCommand::read_from(input).map(Command::Work),
            0x04 => CreateCommand::read_from(input).map(Command::Create),
            0x05 => AddAttributeCommand::read_from(input).map(Command::AddAttribute),
            0x0a => AIOrderCommand::read_from(input).map(Command::AIOrder),
            0x10 => GroupWaypointCommand::read_from(input).map(Command::GroupWaypoint),
            0x12 => UnitAIStateCommand::read_from(input).map(Command::UnitAIState),
            0x35 => UserPatchAICommand::read_from(input, len).map(Command::UserPatchAI),
            0x64 => MakeCommand::read_from(input).map(Command::Make),
            0x65 => ResearchCommand::read_from(input).map(Command::Research),
            0x66 => BuildCommand::read_from(input).map(Command::Build),
            0x67 => GameCommand::read_from(input).map(Command::Game),
            0x69 => BuildWallCommand::read_from(input).map(Command::BuildWall),
            0x6f => UngarrisonCommand::read_from(input).map(Command::Ungarrison),
            0x73 => FlareCommand::read_from(input).map(Command::Flare),
            0x75 => UnitOrderCommand::read_from(input).map(Command::UnitOrder),
            0x80 => BackToWorkCommand::read_from(input).map(Command::BackToWork),
            id => panic!("unsupported command type {:#x}", id),
        };
        let _world_time = input.read_u32::<LE>()?;
        command
    }
}

#[derive(Debug, Default, Clone)]
pub struct Sync {
    pub sequence: Option<u8>,
    pub time: u32,
    pub checksums: Option<(u32, u32, u32)>,
    pub next_world_time: Option<u32>,
}

impl Sync {
    pub fn read_from<R: Read>(
        input: &mut R,
        use_sequence_numbers: bool,
        includes_checksum: bool,
    ) -> Result<Self> {
        let mut sync = Self::default();
        sync.sequence = if use_sequence_numbers {
            Some(input.read_u8()?)
        } else {
            None
        };
        sync.time = input.read_u32::<LE>()?;
        if false {
            let _old_world_time = input.read_u32::<LE>()?;
            let _unknown = input.read_u32::<LE>()?;
        }
        if includes_checksum {
            let check_bytes = input.read_u32::<LE>()?;
            if check_bytes == 0 {
                let _always_zero = input.read_u32::<LE>()?;
                let checksum = input.read_u32::<LE>()?;
                let position_checksum = input.read_u32::<LE>()?;
                let action_checksum = input.read_u32::<LE>()?;
                let _always_zero = input.read_u32::<LE>()?;
                sync.next_world_time = Some(input.read_u32::<LE>()?);
                sync.checksums = Some((checksum, position_checksum, action_checksum));
            }
        }
        Ok(sync)
    }
}

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Clone)]
pub struct Chat {
    message: String,
}

impl Chat {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        assert_eq!(input.read_i32::<LE>()?, -1);
        let length = input.read_u32::<LE>()?;
        let mut bytes = vec![0; length as usize];
        input.read_exact(&mut bytes)?;
        if bytes.last() == Some(&0) {
            bytes.pop();
        }
        let message = String::from_utf8(bytes).unwrap();
        Ok(Self { message })
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Command(Command),
    Sync(Sync),
    ViewLock(ViewLock),
    Chat(Chat),
}

fn skip<R: Read>(input: &mut R, bytes: u64) -> Result<()> {
    std::io::copy(&mut input.by_ref().take(bytes), &mut std::io::sink())?;
    Ok(())
}
