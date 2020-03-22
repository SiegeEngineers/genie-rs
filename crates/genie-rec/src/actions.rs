use crate::{ObjectID, PlayerID, Result};
use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{cmp_float, ReadSkipExt, TechID, UnitTypeID};
use std::convert::TryInto;
use std::io::{Read, Write};

/// A location with an X and Y coordinate.
pub type Location2 = (f32, f32);
/// A location with an X, Y, and Z coordinate.
///
/// The Z coordinate is usually meaningless.
pub type Location3 = (f32, f32, f32);

/// A viewpoint update, recording where the player is currently looking.
///
/// This is used for the View Lock feature when watching a game.
#[derive(Debug, Default, Clone)]
pub struct ViewLock {
    /// The X coordinate the player is looking at.
    pub x: f32,
    /// The Y coordinate the player is looking at.
    pub y: f32,
    /// The ID of the POV player.
    pub player: PlayerID,
}

impl ViewLock {
    /// Read a view lock action from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let x = input.read_f32::<LE>()?;
        let y = input.read_f32::<LE>()?;
        let player = input.read_i32::<LE>()?.try_into().unwrap();
        Ok(Self { x, y, player })
    }

    /// Write a view lock action to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_f32::<LE>(self.x)?;
        output.write_f32::<LE>(self.y)?;
        output.write_i32::<LE>(self.player.try_into().unwrap())?;
        Ok(())
    }
}

/// A list of objects that a command applies to.
///
/// The game uses a special value if a command applies to the same objects as the previous command.
/// That way it does not have to resend 40 object IDs every time a player moves their army. It's
/// encoded as `ObjectsList::SameAsLast` here.
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
    /// Read a list of objects from an input stream.
    pub fn read_from(mut input: impl Read, count: i32) -> Result<Self> {
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

    /// Write a list of objects to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        if let ObjectsList::List(list) = self {
            for entry in list.iter().cloned() {
                output.write_u32::<LE>(entry.into())?;
            }
        }
        Ok(())
    }

    /// The amount of objects contained in this list.
    ///
    /// For `ObjectsList::SameAsLast` this returns 0.
    pub fn len(&self) -> usize {
        match self {
            ObjectsList::SameAsLast => 0,
            ObjectsList::List(list) => list.len(),
        }
    }

    /// Does this list contain 0 objects?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Task an object to a target object or a target location.
#[derive(Debug, Default, Clone)]
pub struct OrderCommand {
    /// The ID of the player executing this command.
    pub player_id: PlayerID,
    /// The target object of this order.
    pub target_id: Option<ObjectID>,
    /// The target location of this order.
    pub location: Location2,
    /// The objects this command applies to.
    pub objects: ObjectsList,
}

impl OrderCommand {
    /// Read an Order command from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        command.player_id = input.read_u8()?.into();
        input.skip(2)?;
        command.target_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        let selected_count = input.read_i32::<LE>()?;
        command.location = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        command.objects = ObjectsList::read_from(input, selected_count)?;
        Ok(command)
    }

    /// Write an Order command to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.player_id.into())?;
        output.write_all(&[0, 0])?;
        output.write_i32::<LE>(self.target_id.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_u32::<LE>(self.objects.len().try_into().unwrap())?;
        output.write_f32::<LE>(self.location.0)?;
        output.write_f32::<LE>(self.location.1)?;
        self.objects.write_to(output)?;
        Ok(())
    }
}

/// Task objects to stop.
#[derive(Debug, Default, Clone)]
pub struct StopCommand {
    /// The objects to stop.
    pub objects: ObjectsList,
}

impl StopCommand {
    /// Read a Stop command from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = input.read_i8()?;
        command.objects = ObjectsList::read_from(input, selected_count as i32)?;
        Ok(command)
    }

    /// Write this Stop command to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i8(self.objects.len().try_into().unwrap())?;
        self.objects.write_to(output)?;
        Ok(())
    }
}

/// Task an object to work.
#[derive(Debug, Default, Clone)]
pub struct WorkCommand {
    /// The target object of this command.
    pub target_id: Option<ObjectID>,
    /// The target location of this command.
    pub location: Location2,
    /// The objects being tasked.
    pub objects: ObjectsList,
}

impl WorkCommand {
    /// Read a Work command from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        input.skip(3)?;
        command.target_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        let selected_count = input.read_i8()?;
        input.skip(3)?;
        command.location = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        command.objects = ObjectsList::read_from(input, selected_count as i32)?;
        Ok(command)
    }

    /// Write this Work command to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_all(&[0, 0, 0])?;
        output.write_i32::<LE>(self.target_id.map(|u| u32::from(u) as i32).unwrap_or(-1))?;
        output.write_i8(self.objects.len().try_into().unwrap())?;
        output.write_all(&[0, 0, 0])?;
        output.write_f32::<LE>(self.location.0)?;
        output.write_f32::<LE>(self.location.1)?;
        self.objects.write_to(output)?;
        Ok(())
    }
}

/// Task an object to move.
#[derive(Debug, Default, Clone)]
pub struct MoveCommand {
    /// The ID of the player issuing this command.
    pub player_id: PlayerID,
    /// The target object of this command.
    pub target_id: Option<ObjectID>,
    /// The target location of this command.
    pub location: Location2,
    /// The objects being tasked.
    pub objects: ObjectsList,
}

impl MoveCommand {
    /// Read a Move command from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        command.player_id = input.read_u8()?.into();
        input.skip(2)?;
        command.target_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        let selected_count = input.read_i8()?;
        input.skip(3)?;
        command.location = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        command.objects = ObjectsList::read_from(input, selected_count as i32)?;
        Ok(command)
    }

    /// Write this Move command to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_all(&[0, 0, 0])?;
        output.write_i32::<LE>(self.target_id.map(|u| u32::from(u) as i32).unwrap_or(-1))?;
        output.write_i8(self.objects.len().try_into().unwrap())?;
        output.write_all(&[0, 0, 0])?;
        output.write_f32::<LE>(self.location.0)?;
        output.write_f32::<LE>(self.location.1)?;
        self.objects.write_to(output)?;
        Ok(())
    }
}
/// A command that instantly places a unit type at a given location.
///
/// Typically used for cheats and the like.
#[derive(Debug, Default, Clone)]
pub struct CreateCommand {
    /// The ID of the player issuing this command.
    pub player_id: PlayerID,
    /// The type of unit to create.
    pub unit_type_id: UnitTypeID,
    /// The location.
    pub location: Location3,
}

impl CreateCommand {
    /// Read a Create command from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        let _padding = input.read_u8()?;
        command.unit_type_id = input.read_u16::<LE>()?.into();
        command.player_id = input.read_u8()?.into();
        let _padding = input.read_u8()?;
        command.location = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        Ok(command)
    }

    /// Write this Create command to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(0)?;
        output.write_u16::<LE>(self.unit_type_id.into())?;
        output.write_u8(self.player_id.into())?;
        output.write_u8(0)?;
        output.write_f32::<LE>(self.location.0)?;
        output.write_f32::<LE>(self.location.1)?;
        output.write_f32::<LE>(self.location.2)?;
        Ok(())
    }
}

/// Add resources to a player's stockpile.
///
/// Typically used for cheats.
#[derive(Debug, Default, Clone)]
pub struct AddResourceCommand {
    /// The player this command applies to.
    pub player_id: PlayerID,
    /// The resource to add.
    pub resource: u8,
    /// The amount to add to this resource. May be negative for subtracting.
    pub amount: f32,
}

impl AddResourceCommand {
    /// Read an AddResource command from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let player_id = input.read_u8()?.into();
        let resource = input.read_u8()?;
        let _padding = input.read_u8()?;
        let amount = input.read_f32::<LE>()?;
        Ok(Self {
            player_id,
            resource,
            amount,
        })
    }

    /// Write this AddResource command to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.player_id.into())?;
        output.write_u8(self.resource)?;
        output.write_u8(0)?;
        output.write_f32::<LE>(self.amount)?;
        Ok(())
    }
}

///
#[derive(Debug, Default, Clone)]
pub struct AIOrderCommand {
    pub player_id: PlayerID,
    pub issuer: PlayerID,
    pub objects: ObjectsList,
    pub order_type: u16,
    pub order_priority: i8,
    pub target_id: Option<ObjectID>,
    pub target_player_id: Option<PlayerID>,
    pub target_location: Location3,
    pub range: f32,
    pub immediate: bool,
    pub add_to_front: bool,
}

impl AIOrderCommand {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
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
        input.skip(3)?;
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

/// A player resigns or drops from the game.
#[derive(Debug, Default, Clone)]
pub struct ResignCommand {
    /// The ID of the player that is resigning.
    pub player_id: PlayerID,
    /// The multiplayer ID of the player that is resigning.
    pub comm_player_id: PlayerID,
    /// Is this "resignation" because the player dropped from the game?
    pub dropped: bool,
}

impl ResignCommand {
    /// Read a Resign command from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let player_id = input.read_u8()?.into();
        let comm_player_id = input.read_u8()?.into();
        let dropped = input.read_u8()? != 0;
        Ok(Self {
            player_id,
            comm_player_id,
            dropped,
        })
    }

    /// Write this Resign command to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.player_id.into())?;
        output.write_u8(self.comm_player_id.into())?;
        output.write_u8(if self.dropped { 1 } else { 0 })?;
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
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let player_id = input.read_u8()?.into();
        input.skip(2)?;
        let object_id = input.read_u32::<LE>()?.into();
        let waypoints = input.read_i8()?;
        input.skip(1)?;
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

/// Set a group of objects's "AI State" (usually known as "stance").
#[derive(Debug, Default, Clone)]
pub struct UnitAIStateCommand {
    /// The new state. Aggressive/Defensive/No Attack/ etc.
    pub state: i8,
    /// The objects whose AI state is being changed.
    pub objects: ObjectsList,
}

impl UnitAIStateCommand {
    /// Read a UnitAIState command from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let selected_count = input.read_u8()?;
        let state = input.read_i8()?;
        let objects = ObjectsList::read_from(input, i32::from(selected_count))?;
        Ok(Self { state, objects })
    }

    /// Write this UnitAIState command to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.objects.len().try_into().unwrap())?;
        output.write_i8(self.state)?;
        self.objects.write_to(output)?;
        Ok(())
    }
}

/// Task units to guard an object.
#[derive(Debug, Default, Clone)]
pub struct GuardCommand {
    /// The target object of this order.
    pub target_id: Option<ObjectID>,
    /// The objects this command applies to.
    pub objects: ObjectsList,
}

impl GuardCommand {
    /// Read a Guard command from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = i32::from(input.read_u8()?);
        input.skip(2)?;
        command.target_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        command.objects = ObjectsList::read_from(input, selected_count)?;
        Ok(command)
    }

    /// Write a Guard command to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.objects.len().try_into().unwrap())?;
        output.write_all(&[0, 0])?;
        output.write_i32::<LE>(self.target_id.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        self.objects.write_to(output)?;
        Ok(())
    }
}

/// Task units to follow an object.
#[derive(Debug, Default, Clone)]
pub struct FollowCommand {
    /// The target object of this order.
    pub target_id: Option<ObjectID>,
    /// The objects this command applies to.
    pub objects: ObjectsList,
}

impl FollowCommand {
    /// Read a Follow command from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = i32::from(input.read_u8()?);
        input.skip(2)?;
        command.target_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        command.objects = ObjectsList::read_from(input, selected_count)?;
        Ok(command)
    }

    /// Write a Follow command to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.objects.len().try_into().unwrap())?;
        output.write_all(&[0, 0])?;
        output.write_i32::<LE>(self.target_id.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        self.objects.write_to(output)?;
        Ok(())
    }
}

/// Task a group of objects to patrol along a given path.
#[derive(Debug, Default, Clone)]
pub struct PatrolCommand {
    /// The waypoints that this patrol should pass through.
    pub waypoints: ArrayVec<[Location2; 10]>,
    /// The objects to include in this formation.
    pub objects: ObjectsList,
}

impl PatrolCommand {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = input.read_i8()?;
        let waypoint_count = input.read_u8()?;
        let _padding = input.read_u8()?;
        let mut raw_waypoints = [(0.0, 0.0); 10];
        for w in raw_waypoints.iter_mut() {
            w.0 = input.read_f32::<LE>()?;
        }
        for w in raw_waypoints.iter_mut() {
            w.1 = input.read_f32::<LE>()?;
        }
        command
            .waypoints
            .try_extend_from_slice(&raw_waypoints[0..usize::from(waypoint_count)])
            .unwrap();
        command.objects = ObjectsList::read_from(input, i32::from(selected_count))?;
        Ok(command)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i8(self.objects.len().try_into().unwrap())?;
        output.write_u8(self.waypoints.len().try_into().unwrap())?;
        output.write_u8(0)?;
        for i in 0..10 {
            output.write_f32::<LE>(self.waypoints.get(i).cloned().unwrap_or_default().0)?;
        }
        for i in 0..10 {
            output.write_f32::<LE>(self.waypoints.get(i).cloned().unwrap_or_default().1)?;
        }
        self.objects.write_to(output)?;
        Ok(())
    }
}

/// Task a group of objects to form a formation.
#[derive(Debug, Default, Clone)]
pub struct FormFormationCommand {
    /// The ID of the player issuing this command.
    pub player_id: PlayerID,
    /// The type of formation to form.
    pub formation_type: i32,
    /// The objects to include in this formation.
    pub objects: ObjectsList,
}

impl FormFormationCommand {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = input.read_i8()?;
        command.player_id = input.read_u8()?.into();
        let _padding = input.read_u8()?;
        command.formation_type = input.read_i32::<LE>()?;
        command.objects = ObjectsList::read_from(input, i32::from(selected_count))?;
        Ok(command)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i8(self.objects.len().try_into().unwrap())?;
        output.write_u8(self.player_id.into())?;
        output.write_u8(0)?;
        output.write_i32::<LE>(self.formation_type)?;
        self.objects.write_to(output)?;
        Ok(())
    }
}

/// Meta-command for UserPatch's new AI commands.
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
    pub fn read_from(mut input: impl Read, size: u32) -> Result<Self> {
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
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        input.skip(3)?;
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

/// Start a research.
#[derive(Debug, Default, Clone)]
pub struct ResearchCommand {
    /// The ID of the player starting the research.
    pub player_id: PlayerID,
    /// The building where the research is taking place.
    pub building_id: ObjectID,
    /// The tech ID of the research.
    pub tech_id: TechID,
    /// TODO
    pub target_id: Option<ObjectID>,
}

impl ResearchCommand {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        input.skip(3)?;
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

/// Place a building foundation and task a group of villagers to start building.
#[derive(Debug, Default, Clone)]
pub struct BuildCommand {
    /// The ID of the player issuing this command.
    pub player_id: PlayerID,
    /// The type of building to place.
    pub unit_type_id: UnitTypeID,
    /// The location of the new building foundation.
    pub location: Location2,
    /// The index of the frame to use, for buildings with multiple graphics like houses.
    pub frame: u8,
    /// The IDs of the villagers that are tasked to build this building.
    pub builders: ObjectsList,
    /// Unique ID for the _command_ (not building)? Used by AIs?
    unique_id: Option<u32>,
}

impl BuildCommand {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
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
        input.skip(3)?;
        command.builders = ObjectsList::read_from(input, i32::from(selected_count))?;
        Ok(command)
    }
}

/// Commands affecting the game.
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
    pub fn read_from(mut input: impl Read) -> Result<Self> {
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
    pub fn read_from(mut input: impl Read) -> Result<Self> {
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

/// Task a group of villagers to build a wall from point A to point B.
#[derive(Debug, Default, Clone)]
pub struct BuildWallCommand {
    pub player_id: PlayerID,
    pub start: (u8, u8),
    pub end: (u8, u8),
    pub unit_type_id: UnitTypeID,
    pub builders: ObjectsList,
}

impl BuildWallCommand {
    fn read_from(mut input: impl Read) -> Result<Self> {
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

/// Delete a building or cancel a building that's not fully built yet.
#[derive(Debug, Default, Clone)]
pub struct CancelBuildCommand {
    /// The ID of the player issuing this command.
    pub player_id: PlayerID,
    /// The ID of the building to cancel.
    pub building_id: ObjectID,
}

impl CancelBuildCommand {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        input.skip(3)?;
        let building_id = input.read_u32::<LE>()?.into();
        let player_id = input.read_u32::<LE>()?.try_into().unwrap();
        Ok(Self {
            player_id,
            building_id,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_all(&[0, 0, 0])?;
        output.write_u32::<LE>(self.building_id.into())?;
        output.write_u32::<LE>(self.player_id.try_into().unwrap())?;
        Ok(())
    }
}

/// Ungarrison objects from a given list of objects.
#[derive(Debug, Default, Clone)]
pub struct UngarrisonCommand {
    pub ungarrison_type: i8,
    pub unit_type_id: Option<ObjectID>,
    pub location: Option<Location2>,
    pub objects: ObjectsList,
}

impl UngarrisonCommand {
    fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = input.read_i8()?;
        let _padding = input.read_u16::<LE>()?;
        let x = input.read_f32::<LE>()?;
        let y = input.read_f32::<LE>()?;
        command.location = if cmp_float!(x != -1.0) && cmp_float!(y != -1.0) {
            Some((x, y))
        } else {
            None
        };
        command.ungarrison_type = input.read_i8()?;
        input.skip(3)?;
        command.unit_type_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        command.objects = ObjectsList::read_from(input, i32::from(selected_count))?;
        Ok(command)
    }
}

/// Send a flare at the given location.
#[derive(Debug, Default, Clone)]
pub struct FlareCommand {
    pub player_id: PlayerID,
    pub comm_player_id: PlayerID,
    pub recipients: [bool; 9],
    pub location: Location2,
}

impl FlareCommand {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        input.skip(3)?;
        assert_eq!(
            input.read_i32::<LE>()?,
            -1,
            "found flare with unexpected unit id"
        );
        for receive in command.recipients.iter_mut() {
            *receive = input.read_u8()? != 0;
        }
        input.skip(3)?;
        command.location = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        command.player_id = input.read_u8()?.into();
        command.comm_player_id = input.read_u8()?.into();
        input.skip(2)?;
        Ok(command)
    }
}

///
#[derive(Debug, Default, Clone)]
pub struct UnitOrderCommand {
    pub target_id: Option<ObjectID>,
    pub action: i8,
    pub param: Option<u8>,
    pub location: Option<Location2>,
    pub unique_id: Option<u32>,
    pub objects: ObjectsList,
}

impl UnitOrderCommand {
    fn read_from(mut input: impl Read) -> Result<Self> {
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
        command.location = if cmp_float!(x != -1.0) && cmp_float!(y != -1.0) {
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

///
#[derive(Debug, Default, Clone)]
pub struct QueueCommand {
    /// The ID of the building where this unit is being queued.
    pub building_id: ObjectID,
    /// The ID of the unit type being queued.
    pub unit_type_id: UnitTypeID,
    /// The amount of units to queue.
    pub amount: u16,
}

impl QueueCommand {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        input.skip(3)?;
        command.building_id = input.read_u32::<LE>()?.into();
        command.unit_type_id = input.read_u16::<LE>()?.into();
        command.amount = input.read_u16::<LE>()?;
        Ok(command)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_all(&[0, 0, 0])?;
        output.write_u32::<LE>(self.building_id.into())?;
        output.write_u16::<LE>(self.unit_type_id.into())?;
        output.write_u16::<LE>(self.amount)?;
        Ok(())
    }
}

///
#[derive(Debug, Default, Clone)]
pub struct SetGatherPointCommand {
    /// The IDs of the buildings whose gather points are being set.
    pub buildings: ObjectsList,
    /// The ID of the object being targeted, if the gather point is set to an object.
    pub target_id: Option<ObjectID>,
    /// The type ID of the unit being targeted, if the gather point is set to an object.
    pub target_type_id: Option<UnitTypeID>,
    /// The location of the new gather point, if the gather point is not set to an object.
    pub location: Option<Location2>,
}

impl SetGatherPointCommand {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut command = Self::default();
        let selected_count = i32::from(input.read_i8()?);
        input.skip(2)?;
        command.target_id = match input.read_i16::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        command.target_type_id = match input.read_i16::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        input.skip(2)?;
        command.location = Some((
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        ));
        command.buildings = ObjectsList::read_from(input, selected_count)?;
        Ok(command)
    }

    pub fn write_to<W: Write>(&self, _output: &mut W) -> Result<()> {
        todo!()
    }
}

/// Read and write impl for market buying/selling commands, which are different commands but have
/// the same shape.
macro_rules! buy_sell_impl {
    ($name:ident) => {
        impl $name {
            pub fn read_from(mut input: impl Read) -> Result<Self> {
                let mut command = Self::default();
                command.player_id = input.read_u8()?.into();
                command.resource = input.read_u8()?;
                command.amount = input.read_i8()?;
                command.market_id = input.read_u32::<LE>()?.into();
                Ok(command)
            }

            pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
                output.write_u8(self.player_id.into())?;
                output.write_u8(self.resource)?;
                output.write_i8(self.amount)?;
                output.write_u32::<LE>(self.market_id.into())?;
                Ok(())
            }
        }
    };
}

/// Sell a resource at the market.
#[derive(Debug, Default, Clone)]
pub struct SellResourceCommand {
    /// The ID of the player issuing this command.
    pub player_id: PlayerID,
    /// The resource being sold.
    pub resource: u8,
    /// The amount being sold, in 100s. Typically this is 1 for selling 100 of a resource, or 5 for
    /// selling 500 (with Shift-click).
    pub amount: i8,
    /// The ID of the building where this resource is being bought.
    pub market_id: ObjectID,
}

buy_sell_impl!(SellResourceCommand);

/// Buy a resource at the market.
#[derive(Debug, Default, Clone)]
pub struct BuyResourceCommand {
    /// The ID of the player issuing this command.
    pub player_id: PlayerID,
    /// The resource being bought.
    pub resource: u8,
    /// The amount being bought, in 100s. Typically this is 1 for buying 100 of a resource, or 5 for
    /// buying 500 (with Shift-click).
    pub amount: i8,
    /// The ID of the building where this resource is being bought.
    pub market_id: ObjectID,
}

buy_sell_impl!(BuyResourceCommand);

/// Send villagers back to work after they've been garrisoned into the Town Center.
#[derive(Debug, Default, Clone)]
pub struct BackToWorkCommand {
    pub building_id: ObjectID,
}

impl BackToWorkCommand {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        input.skip(3)?;
        let building_id = input.read_u32::<LE>()?.into();
        Ok(Self { building_id })
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Order(OrderCommand),
    Stop(StopCommand),
    Work(WorkCommand),
    Move(MoveCommand),
    Create(CreateCommand),
    AddResource(AddResourceCommand),
    AIOrder(AIOrderCommand),
    Resign(ResignCommand),
    GroupWaypoint(GroupWaypointCommand),
    UnitAIState(UnitAIStateCommand),
    Guard(GuardCommand),
    Follow(FollowCommand),
    Patrol(PatrolCommand),
    FormFormation(FormFormationCommand),
    UserPatchAI(UserPatchAICommand),
    Make(MakeCommand),
    Research(ResearchCommand),
    Build(BuildCommand),
    Game(GameCommand),
    BuildWall(BuildWallCommand),
    CancelBuild(CancelBuildCommand),
    Ungarrison(UngarrisonCommand),
    Flare(FlareCommand),
    UnitOrder(UnitOrderCommand),
    Queue(QueueCommand),
    SetGatherPoint(SetGatherPointCommand),
    SellResource(SellResourceCommand),
    BuyResource(BuyResourceCommand),
    BackToWork(BackToWorkCommand),
}

impl Command {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let len = input.read_u32::<LE>()?;
        let mut small_buffer;
        let mut big_buffer;
        let buffer: &mut [u8] = if len > 512 {
            small_buffer = [0; 512];
            &mut small_buffer
        } else {
            big_buffer = vec![0; len as usize];
            &mut big_buffer
        };

        input.read_exact(buffer)?;
        let mut cursor = std::io::Cursor::new(buffer);
        let command = match cursor.read_u8()? {
            0x00 => OrderCommand::read_from(cursor).map(Command::Order),
            0x01 => StopCommand::read_from(cursor).map(Command::Stop),
            0x02 => WorkCommand::read_from(cursor).map(Command::Work),
            0x03 => MoveCommand::read_from(cursor).map(Command::Move),
            0x04 => CreateCommand::read_from(cursor).map(Command::Create),
            0x05 => AddResourceCommand::read_from(cursor).map(Command::AddResource),
            0x0a => AIOrderCommand::read_from(cursor).map(Command::AIOrder),
            0x0b => ResignCommand::read_from(cursor).map(Command::Resign),
            0x10 => GroupWaypointCommand::read_from(cursor).map(Command::GroupWaypoint),
            0x12 => UnitAIStateCommand::read_from(cursor).map(Command::UnitAIState),
            0x13 => GuardCommand::read_from(cursor).map(Command::Guard),
            0x14 => FollowCommand::read_from(cursor).map(Command::Follow),
            0x15 => PatrolCommand::read_from(cursor).map(Command::Patrol),
            0x17 => FormFormationCommand::read_from(cursor).map(Command::FormFormation),
            0x35 => UserPatchAICommand::read_from(cursor, len).map(Command::UserPatchAI),
            0x64 => MakeCommand::read_from(cursor).map(Command::Make),
            0x65 => ResearchCommand::read_from(cursor).map(Command::Research),
            0x66 => BuildCommand::read_from(cursor).map(Command::Build),
            0x67 => GameCommand::read_from(cursor).map(Command::Game),
            0x69 => BuildWallCommand::read_from(cursor).map(Command::BuildWall),
            0x6a => CancelBuildCommand::read_from(cursor).map(Command::CancelBuild),
            0x6f => UngarrisonCommand::read_from(cursor).map(Command::Ungarrison),
            0x73 => FlareCommand::read_from(cursor).map(Command::Flare),
            0x75 => UnitOrderCommand::read_from(cursor).map(Command::UnitOrder),
            0x77 => QueueCommand::read_from(cursor).map(Command::Queue),
            0x78 => SetGatherPointCommand::read_from(cursor).map(Command::SetGatherPoint),
            0x7a => SellResourceCommand::read_from(cursor).map(Command::SellResource),
            0x7b => BuyResourceCommand::read_from(cursor).map(Command::BuyResource),
            0x80 => BackToWorkCommand::read_from(cursor).map(Command::BackToWork),
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

/// Action at the start of the game, contains settings affecting the rec format.
#[derive(Debug, Default, Clone)]
pub struct Meta {
    pub checksum_interval: u32,
    pub is_multiplayer: bool,
    pub use_sequence_numbers: bool,
    pub local_player_id: PlayerID,
    pub header_position: u32,
    /// The amount of saved chapters in this rec / save game. This is only set if the game version
    /// that generated the file supports saved chapters (i.e. The Conquerors and up).
    pub num_chapters: Option<u32>,
}

impl Meta {
    /// Read the chunk of recorded game body metadata that's the same across all versions.
    fn read_from_inner(mut input: impl Read) -> Result<Self> {
        let checksum_interval = input.read_u32::<LE>()?;
        let is_multiplayer = input.read_u32::<LE>()? != 0;
        let local_player_id = input.read_u32::<LE>()?.try_into().unwrap();
        let header_position = input.read_u32::<LE>()?;
        let use_sequence_numbers = input.read_u32::<LE>()? != 0;
        Ok(Self {
            checksum_interval,
            is_multiplayer,
            use_sequence_numbers,
            local_player_id,
            header_position,
            ..Default::default()
        })
    }

    /// Read recorded game body metadata in the `mgl` format used by Age of Empires 2: The
    /// Age Of Kings.
    pub fn read_from_mgl(mut input: impl Read) -> Result<Self> {
        let mut meta = Self::read_from_inner(&mut input)?;
        let _exe_file_size = input.read_u64::<LE>()?;
        let _unknown = input.read_f32::<LE>()?;
        let _unknown = input.read_f32::<LE>()?;

        // TODO if `is_multiplayer` flag contains 2 or 3, the `remaining_syncs_until_checksum`
        // value is stored here as u32

        Ok(meta)
    }

    /// Read recorded game body metadata in the `mgx` format used by Age of Empires 2: The
    /// Conquerors and all subsequent versions.
    pub fn read_from_mgx(mut input: impl Read) -> Result<Self> {
        let mut meta = Self::read_from_inner(&mut input)?;
        meta.num_chapters = Some(input.read_u32::<LE>()?);
        Ok(meta)
    }
}

/// A chat message sent during the game.
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

/// An action: TODO
#[derive(Debug, Clone)]
pub enum Action {
    Command(Command),
    Sync(Sync),
    ViewLock(ViewLock),
    Chat(Chat),
}
