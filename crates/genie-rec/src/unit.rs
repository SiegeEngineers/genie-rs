use crate::action::UnitAction;
use crate::unit_type::UnitBaseClass;
use crate::Result;
use crate::{ObjectID, PlayerID};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_dat::UnitType;
pub use genie_dat::{AttributeCost, SpriteID};
use genie_support::read_opt_u32;
pub use genie_support::{StringKey, UnitTypeID};
use std::convert::TryInto;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct Unit {
    pub unit_base_class: UnitBaseClass,
    pub static_: StaticUnitAttributes,
    pub animated: Option<AnimatedUnitAttributes>,
    pub moving: Option<MovingUnitAttributes>,
    pub action: Option<ActionUnitAttributes>,
    pub base_combat: Option<BaseCombatUnitAttributes>,
    pub missile: Option<MissileUnitAttributes>,
    pub combat: Option<CombatUnitAttributes>,
}

impl Unit {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Option<Self>> {
        let raw_class = input.read_u8()?;
        if raw_class == 0 {
            return Ok(None);
        }
        let unit_base_class = raw_class.try_into().unwrap();
        let static_ = StaticUnitAttributes::read_from(&mut input, version)?;
        let mut unit = Self {
            unit_base_class,
            static_,
            animated: None,
            moving: None,
            action: None,
            base_combat: None,
            missile: None,
            combat: None,
        };
        if unit_base_class >= UnitBaseClass::Animated {
            unit.animated = Some(AnimatedUnitAttributes::read_from(&mut input)?);
        }
        if unit_base_class >= UnitBaseClass::Moving {
            unit.moving = Some(MovingUnitAttributes::read_from(&mut input, version)?);
        }
        if unit_base_class >= UnitBaseClass::Action {
            unit.action = Some(ActionUnitAttributes::read_from(&mut input)?);
        }
        if unit_base_class >= UnitBaseClass::BaseCombat {
            unit.base_combat = Some(BaseCombatUnitAttributes::read_from(&mut input)?);
        }
        if unit_base_class >= UnitBaseClass::Missile {
            unit.missile = Some(MissileUnitAttributes::read_from(&mut input, version)?);
        }
        if unit_base_class >= UnitBaseClass::Combat {
            unit.combat = Some(CombatUnitAttributes::read_from(&mut input, version)?);
        }
        Ok(Some(unit))
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        let raw_class = self.unit_base_class as u8;
        output.write_u8(raw_class)?;
        self.static_.write_to(&mut output, version)?;
        if let Some(animated) = &self.animated {
            animated.write_to(&mut output)?;
        }
        if let Some(moving) = &self.moving {
            moving.write_to(&mut output)?;
        }
        if let Some(action) = &self.action {
            action.write_to(&mut output, version)?;
        }
        if let Some(base_combat) = &self.base_combat {
            base_combat.write_to(&mut output, version)?;
        }
        if let Some(missile) = &self.missile {
            missile.write_to(&mut output, version)?;
        }
        if let Some(combat) = &self.combat {
            combat.write_to(&mut output, version)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct SpriteNodeAnimation {
    pub animate_interval: u32,
    pub animate_last: u32,
    pub last_frame: u16,
    pub frame_changed: u8,
    pub frame_looped: u8,
    pub animate_flag: u8,
    pub last_speed: f32,
}

impl SpriteNodeAnimation {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut animation = Self::default();
        animation.animate_interval = input.read_u32::<LE>()?;
        animation.animate_last = input.read_u32::<LE>()?;
        animation.last_frame = input.read_u16::<LE>()?;
        animation.frame_changed = input.read_u8()?;
        animation.frame_looped = input.read_u8()?;
        animation.animate_flag = input.read_u8()?;
        animation.last_speed = input.read_f32::<LE>()?;
        Ok(animation)
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_u32::<LE>(self.animate_interval)?;
        output.write_u32::<LE>(self.animate_last)?;
        output.write_u16::<LE>(self.last_frame)?;
        output.write_u8(self.frame_changed)?;
        output.write_u8(self.frame_looped)?;
        output.write_u8(self.animate_flag)?;
        output.write_f32::<LE>(self.last_speed)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct SpriteNode {
    pub id: SpriteID,
    pub x: u32,
    pub y: u32,
    pub frame: u16,
    pub invisible: bool,
    pub animation: Option<SpriteNodeAnimation>,
    pub order: u8,
    pub flag: u8,
    pub count: u8,
}

impl SpriteNode {
    pub fn read_from(mut input: impl Read) -> Result<Option<Self>> {
        let ty = input.read_u8()?;
        if ty == 0 {
            return Ok(None);
        }

        let mut node = Self::default();
        node.id = input.read_u16::<LE>()?.into();
        node.x = input.read_u32::<LE>()?;
        node.y = input.read_u32::<LE>()?;
        node.frame = input.read_u16::<LE>()?;
        node.invisible = input.read_u8()? != 0;
        if ty == 2 {
            node.animation = Some(SpriteNodeAnimation::read_from(&mut input)?);
        }
        node.order = input.read_u8()?;
        node.flag = input.read_u8()?;
        node.count = input.read_u8()?;
        Ok(Some(node))
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        let ty = if self.animation.is_some() { 2 } else { 1 };
        output.write_u8(ty)?;
        output.write_u16::<LE>(self.id.into())?;
        output.write_u32::<LE>(self.x)?;
        output.write_u32::<LE>(self.y)?;
        output.write_u16::<LE>(self.frame)?;
        output.write_u8(if self.invisible { 1 } else { 0 })?;
        if let Some(animation) = &self.animation {
            animation.write_to(&mut output)?;
        }
        output.write_u8(self.order)?;
        output.write_u8(self.flag)?;
        output.write_u8(self.count)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct SpriteList {
    pub sprites: Vec<SpriteNode>,
}

impl SpriteList {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut sprites = vec![];
        while let Some(node) = SpriteNode::read_from(&mut input)? {
            sprites.push(node);
        }
        Ok(Self { sprites })
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        for sprite in &self.sprites {
            sprite.write_to(&mut output)?;
        }
        output.write_u8(0)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct StaticUnitAttributes {
    pub owner_id: PlayerID,
    pub unit_type_id: UnitTypeID,
    pub sprite_id: SpriteID,
    pub garrisoned_in_id: Option<ObjectID>,
    pub hit_points: f32,
    pub object_state: u8,
    pub sleep_flag: bool,
    pub dopple_flag: bool,
    pub go_to_sleep_flag: bool,
    pub id: ObjectID,
    pub facet: u8,
    pub position: (f32, f32, f32),
    pub screen_offset: (u16, u16),
    pub shadow_offset: (u16, u16),
    pub selected_group: Option<u8>,
    pub attribute_type_held: u16,
    pub attribute_amount_held: f32,
    pub worker_count: u8,
    pub current_damage: u8,
    pub damaged_lately_timer: u8,
    pub under_attack: bool,
    pub pathing_group_members: Vec<ObjectID>,
    pub group_id: Option<u32>,
    pub roo_already_called: u8,
    pub sprite_list: Option<SpriteList>,
}

impl StaticUnitAttributes {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.owner_id = input.read_u8()?.into();
        attrs.unit_type_id = input.read_u16::<LE>()?.into();
        attrs.sprite_id = input.read_u16::<LE>()?.into();
        attrs.garrisoned_in_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        attrs.hit_points = input.read_f32::<LE>()?;
        attrs.object_state = input.read_u8()?;
        attrs.sleep_flag = input.read_u8()? != 0;
        attrs.dopple_flag = input.read_u8()? != 0;
        attrs.go_to_sleep_flag = input.read_u8()? != 0;
        attrs.id = input.read_u32::<LE>()?.into();
        attrs.facet = input.read_u8()?;
        attrs.position = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        attrs.screen_offset = (input.read_u16::<LE>()?, input.read_u16::<LE>()?);
        attrs.shadow_offset = (input.read_u16::<LE>()?, input.read_u16::<LE>()?);
        if version < 11.58 {
            attrs.selected_group = match input.read_i8()? {
                -1 => None,
                id => Some(id.try_into().unwrap()),
            };
        }
        attrs.attribute_type_held = input.read_u16::<LE>()?;
        attrs.attribute_amount_held = input.read_f32::<LE>()?;
        attrs.worker_count = input.read_u8()?;
        attrs.current_damage = input.read_u8()?;
        attrs.damaged_lately_timer = input.read_u8()?;
        attrs.under_attack = input.read_u8()? != 0;
        attrs.pathing_group_members = {
            let num_members = input.read_u32::<LE>()?;
            let mut members = vec![ObjectID(0); num_members.try_into().unwrap()];
            for m in members.iter_mut() {
                *m = input.read_u32::<LE>()?.into();
            }
            members
        };
        attrs.group_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        attrs.roo_already_called = input.read_u8()?;
        if input.read_u8()? != 0 {
            attrs.sprite_list = Some(SpriteList::read_from(&mut input)?);
        }
        Ok(attrs)
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        output.write_u8(self.owner_id.into())?;
        output.write_u16::<LE>(self.unit_type_id.into())?;
        output.write_u16::<LE>(self.sprite_id.into())?;
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct AnimatedUnitAttributes {
    pub speed: f32,
}

impl AnimatedUnitAttributes {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let speed = input.read_f32::<LE>()?;
        Ok(Self { speed })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_f32::<LE>(self.speed)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct PathData {
    pub id: u32,
    pub linked_path_type: u32,
    pub waypoint_level: u32,
    pub path_id: u32,
    pub waypoint: u32,
    pub disable_flags: Option<u32>,
    pub enable_flags: Option<u32>,
    pub state: u32,
    pub range: f32,
    pub target_id: u32,
    pub pause_time: f32,
    pub continue_counter: u32,
    pub flags: u32,
}

impl PathData {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut path = Self::default();
        path.id = input.read_u32::<LE>()?;
        path.linked_path_type = input.read_u32::<LE>()?;
        path.waypoint_level = input.read_u32::<LE>()?;
        path.path_id = input.read_u32::<LE>()?;
        path.waypoint = input.read_u32::<LE>()?;
        if version < 10.25 {
            path.disable_flags = Some(input.read_u32::<LE>()?);
            if version >= 10.20 {
                path.enable_flags = Some(input.read_u32::<LE>()?);
            }
        }
        path.state = input.read_u32::<LE>()?;
        path.range = input.read_f32::<LE>()?;
        path.target_id = input.read_u32::<LE>()?;
        path.pause_time = input.read_f32::<LE>()?;
        path.continue_counter = input.read_u32::<LE>()?;
        path.flags = input.read_u32::<LE>()?;
        Ok(path)
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MovementData {
    pub velocity: (f32, f32, f32),
    pub acceleration: (f32, f32, f32),
}

impl MovementData {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let velocity = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        let acceleration = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        Ok(Self {
            velocity,
            acceleration,
        })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_f32::<LE>(self.velocity.0)?;
        output.write_f32::<LE>(self.velocity.1)?;
        output.write_f32::<LE>(self.velocity.2)?;
        output.write_f32::<LE>(self.acceleration.0)?;
        output.write_f32::<LE>(self.acceleration.1)?;
        output.write_f32::<LE>(self.acceleration.2)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct MovingUnitAttributes {
    pub trail_remainder: u32,
    pub velocity: (f32, f32, f32),
    pub angle: f32,
    pub turn_towards_time: u32,
    pub turn_timer: u32,
    pub continue_counter: u32,
    pub current_terrain_exception: (Option<u32>, Option<u32>),
    pub waiting_to_move: u8,
    pub wait_delays_count: u8,
    pub on_ground: u8,
    pub path_data: Vec<PathData>,
    pub future_path_data: Option<PathData>,
    pub movement_data: Option<MovementData>,
    pub position: (f32, f32, f32),
    pub orientation_forward: (f32, f32, f32),
    pub orientation_right: (f32, f32, f32),
    pub last_move_time: u32,
    pub user_defined_waypoints: Vec<(f32, f32, f32)>,
    pub substitute_position: Option<(f32, f32, f32)>,
    pub consecutive_substitute_count: u32,
}

impl MovingUnitAttributes {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.trail_remainder = input.read_u32::<LE>()?;
        attrs.velocity = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        attrs.angle = input.read_f32::<LE>()?;
        attrs.turn_towards_time = input.read_u32::<LE>()?;
        attrs.turn_timer = input.read_u32::<LE>()?;
        attrs.continue_counter = input.read_u32::<LE>()?;
        attrs.current_terrain_exception = (
            match input.read_i32::<LE>()? {
                -1 => None,
                id => Some(id.try_into().unwrap()),
            },
            match input.read_i32::<LE>()? {
                -1 => None,
                id => Some(id.try_into().unwrap()),
            },
        );
        attrs.waiting_to_move = input.read_u8()?;
        attrs.wait_delays_count = input.read_u8()?;
        attrs.on_ground = input.read_u8()?;
        attrs.path_data = {
            let num_paths = input.read_u32::<LE>()?;
            let mut paths = vec![];
            for _ in 0..num_paths {
                paths.push(PathData::read_from(&mut input, version)?);
            }
            paths
        };
        if input.read_u32::<LE>()? != 0 {
            attrs.future_path_data = Some(PathData::read_from(&mut input, version)?);
        }
        if input.read_u32::<LE>()? != 0 {
            attrs.movement_data = Some(MovementData::read_from(&mut input)?);
        }
        attrs.position = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        attrs.orientation_forward = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        attrs.orientation_right = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        attrs.last_move_time = input.read_u32::<LE>()?;
        attrs.user_defined_waypoints = {
            let num_waypoints = input.read_i32::<LE>()?.max(0);
            let mut waypoints = vec![];
            for _ in 0..num_waypoints {
                waypoints.push((
                    input.read_f32::<LE>()?,
                    input.read_f32::<LE>()?,
                    input.read_f32::<LE>()?,
                ));
            }
            waypoints
        };
        attrs.substitute_position = {
            let exists = input.read_u32::<LE>()? != 0;
            let x = input.read_f32::<LE>()?;
            let y = input.read_f32::<LE>()?;
            let z = input.read_f32::<LE>()?;
            if exists {
                Some((x, y, z))
            } else {
                None
            }
        };
        attrs.consecutive_substitute_count = input.read_u32::<LE>()?;
        Ok(attrs)
    }

    pub fn write_to(&self, _output: impl Write) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ActionUnitAttributes {
    pub waiting: bool,
    pub command_flag: u8,
    pub selected_group_info: u16,
    pub actions: Vec<UnitAction>,
}

impl ActionUnitAttributes {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.waiting = input.read_u8()? != 0;
        attrs.command_flag = input.read_u8()?;
        attrs.selected_group_info = input.read_u16::<LE>()?;
        attrs.actions = UnitAction::read_list_from(input)?;
        Ok(attrs)
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct BaseCombatUnitAttributes {
    pub formation_id: u8,
    pub formation_row: u8,
    pub formation_column: u8,
    pub attack_timer: f32,
    pub capture_flag: u8,
    pub multi_unified_points: u8,
    pub large_object_radius: u8,
    pub attack_count: u32,
}

impl BaseCombatUnitAttributes {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.formation_id = input.read_u8()?;
        attrs.formation_row = input.read_u8()?;
        attrs.formation_column = input.read_u8()?;
        attrs.attack_timer = input.read_f32::<LE>()?;
        attrs.capture_flag = input.read_u8()?;
        attrs.multi_unified_points = input.read_u8()?;
        attrs.large_object_radius = input.read_u8()?;
        attrs.attack_count = input.read_u32::<LE>()?;
        Ok(attrs)
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct MissileUnitAttributes {
    pub max_range: f32,
    pub fired_from_id: ObjectID,
    pub own_base: Option<UnitType>,
}

impl MissileUnitAttributes {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.max_range = input.read_f32::<LE>()?;
        attrs.fired_from_id = input.read_u32::<LE>()?.into();
        attrs.own_base = {
            if input.read_u8()? == 0 {
                None
            } else {
                Some(UnitType::read_from(&mut input, version)?)
            }
        };
        Ok(attrs)
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct UnitAIOrder {
    issuer: u32,
    order_type: u32,
    priority: u32,
    target_id: ObjectID,
    target_player: PlayerID,
    target_location: (f32, f32, f32),
    range: f32,
}

impl UnitAIOrder {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut order = Self::default();
        order.issuer = input.read_u32::<LE>()?;
        order.order_type = input.read_u32::<LE>()?;
        order.priority = input.read_u32::<LE>()?;
        order.target_id = input.read_u32::<LE>()?.into();
        order.target_player = input.read_u32::<LE>()?.try_into().unwrap();
        order.target_location = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        order.range = input.read_f32::<LE>()?;
        Ok(order)
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct UnitAINotification {
    pub caller: u32,
    pub recipient: u32,
    pub notification_type: u32,
    pub params: (u32, u32, u32),
}

impl UnitAINotification {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut notify = Self::default();
        notify.caller = input.read_u32::<LE>()?;
        notify.recipient = input.read_u32::<LE>()?;
        notify.notification_type = input.read_u32::<LE>()?;
        notify.params = (
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
        );
        Ok(notify)
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct UnitAIOrderHistory {
    order: u32,
    action: u32,
    time: u32,
    position: (f32, f32, f32),
    target_id: ObjectID,
    target_attack_category: u32,
    target_position: (f32, f32, f32),
}

impl UnitAIOrderHistory {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut order = Self::default();
        order.order = input.read_u32::<LE>()?;
        order.action = input.read_u32::<LE>()?;
        order.time = input.read_u32::<LE>()?;
        order.position = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        order.target_id = input.read_u32::<LE>()?.into();
        order.target_attack_category = input.read_u32::<LE>()?;
        order.target_position = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        Ok(order)
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UnitAIRetargetEntry {
    pub target_id: ObjectID,
    pub retarget_timeout: u32,
}

impl UnitAIRetargetEntry {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let target_id = input.read_u32::<LE>()?.into();
        let retarget_timeout = input.read_u32::<LE>()?;
        Ok(Self {
            target_id,
            retarget_timeout,
        })
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Waypoint {
    pub location: (f32, f32, f32),
    pub facet_to_next_waypoint: u8,
}

impl Waypoint {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut waypoint = Self::default();
        waypoint.location = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        waypoint.facet_to_next_waypoint = input.read_u8()?;
        let _padding = input.read_u8()?;
        let _padding = input.read_u8()?;
        let _padding = input.read_u8()?;
        Ok(waypoint)
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PatrolPath {}

impl PatrolPath {
    pub fn read_from(_input: impl Read) -> Result<Self> {
        todo!()
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct UnitAI {
    mood: Option<u32>,
    current_order: Option<u32>,
    current_order_priority: Option<u32>,
    current_action: Option<u32>,
    current_target: Option<u32>,
    current_target_type: Option<u32>,
    current_target_location: (f32, f32, f32),
    desired_target_distance: f32,
    last_action: Option<u32>,
    last_order: Option<u32>,
    last_target: Option<u32>,
    last_target_type: Option<u32>,
    last_update_type: Option<u32>,
    idle_timer: u32,
    idle_timeout: u32,
    adjusted_idle_timeout: u32,
    secondary_timer: u32,
    lookaround_timer: u32,
    lookaround_timeout: u32,
    defend_target: Option<ObjectID>,
    defense_buffer: f32,
    last_world_position: Waypoint,
    orders: Vec<UnitAIOrder>,
    notifications: Vec<UnitAINotification>,
    attacking_units: Vec<ObjectID>,
    stop_after_target_killed: bool,
    state: u8,
    state_position: (f32, f32),
    time_since_enemy_sighting: u32,
    alert_mode: u8,
    alert_mode_object_id: Option<ObjectID>,
    patrol_path: Option<PatrolPath>,
    patrol_current_waypoint: u32,
    order_history: Vec<UnitAIOrderHistory>,
    last_retarget_time: u32,
    randomized_retarget_timer: u32,
    retarget_entries: Vec<UnitAIRetargetEntry>,
    best_unit_to_attack: Option<u32>,
    formation_type: u8,
}

impl UnitAI {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut ai = Self::default();
        ai.mood = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        ai.current_order = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        ai.current_order_priority = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        ai.current_action = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        ai.current_target = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        ai.current_target_type = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        ai.current_target_location = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        ai.desired_target_distance = input.read_f32::<LE>()?;
        ai.last_action = read_opt_u32(&mut input)?;
        ai.last_order = read_opt_u32(&mut input)?;
        ai.last_target = read_opt_u32(&mut input)?;
        ai.last_target_type = read_opt_u32(&mut input)?;
        ai.last_update_type = read_opt_u32(&mut input)?;
        ai.idle_timer = input.read_u32::<LE>()?;
        ai.idle_timeout = input.read_u32::<LE>()?;
        ai.adjusted_idle_timeout = input.read_u32::<LE>()?;
        ai.secondary_timer = input.read_u32::<LE>()?;
        ai.lookaround_timer = input.read_u32::<LE>()?;
        ai.lookaround_timeout = input.read_u32::<LE>()?;
        ai.defend_target = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        ai.defense_buffer = input.read_f32::<LE>()?;
        ai.last_world_position = Waypoint::read_from(&mut input)?;
        ai.orders = {
            let num_orders = input.read_u32::<LE>()?;
            let mut orders = vec![];
            for _ in 0..num_orders {
                orders.push(UnitAIOrder::read_from(&mut input)?);
            }
            orders
        };
        ai.notifications = {
            let num_notifications = input.read_u32::<LE>()?;
            let mut notifications = vec![];
            for _ in 0..num_notifications {
                notifications.push(UnitAINotification::read_from(&mut input)?);
            }
            notifications
        };
        ai.attacking_units = {
            let num_units = input.read_u32::<LE>()?;
            let mut units = vec![];
            for _ in 0..num_units {
                units.push(input.read_u32::<LE>()?.into());
            }
            units
        };
        ai.stop_after_target_killed = input.read_u8()? != 0;
        ai.state = input.read_u8()?;
        ai.state_position = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        ai.time_since_enemy_sighting = input.read_u32::<LE>()?;
        ai.alert_mode = input.read_u8()?;
        ai.alert_mode_object_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        ai.patrol_path = {
            let has_path = input.read_u32::<LE>()? != 0;
            if has_path {
                Some(PatrolPath::read_from(&mut input)?)
            } else {
                None
            }
        };
        ai.patrol_current_waypoint = input.read_u32::<LE>()?;
        ai.order_history = {
            let num_orders = input.read_u32::<LE>()?;
            let mut orders = vec![];
            for _ in 0..num_orders {
                orders.push(UnitAIOrderHistory::read_from(&mut input)?);
            }
            orders
        };
        ai.last_retarget_time = input.read_u32::<LE>()?;
        ai.randomized_retarget_timer = input.read_u32::<LE>()?;
        ai.retarget_entries = {
            let num_entries = input.read_u32::<LE>()?;
            let mut entries = vec![];
            for _ in 0..num_entries {
                entries.push(UnitAIRetargetEntry::read_from(&mut input)?);
            }
            entries
        };
        ai.best_unit_to_attack = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        ai.formation_type = input.read_u8()?;
        Ok(ai)
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct CombatUnitAttributes {
    pub next_volley: u8,
    pub using_special_attack_animation: u8,
    pub own_base: Option<UnitType>,
    pub attribute_amounts: [u16; 6],
    pub decay_timer: u16,
    pub raider_build_countdown: u32,
    pub locked_down_count: u32,
    pub inside_garrison_count: u8,
    pub unit_ai: Option<UnitAI>,
    pub town_bell_flag: i8,
    pub town_bell_target_id: Option<ObjectID>,
    pub town_bell_target_location: Option<(f32, f32)>,
    pub town_bell_target_id_2: Option<ObjectID>,
    pub town_bell_target_type: u32,
    pub town_bell_action: u32,
    pub berserker_timer: f32,
    pub num_builders: u8,
    pub num_healers: u8,
}

impl CombatUnitAttributes {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.next_volley = input.read_u8()?;
        attrs.using_special_attack_animation = input.read_u8()?;
        attrs.own_base = {
            if input.read_u8()? == 0 {
                None
            } else {
                Some(UnitType::read_from(&mut input, version)?)
            }
        };
        for amount in attrs.attribute_amounts.iter_mut() {
            *amount = input.read_u16::<LE>()?;
        }
        attrs.decay_timer = input.read_u16::<LE>()?;
        attrs.raider_build_countdown = input.read_u32::<LE>()?;
        attrs.locked_down_count = input.read_u32::<LE>()?;
        attrs.inside_garrison_count = input.read_u8()?;
        attrs.unit_ai = {
            let has_ai = input.read_u32::<LE>()? != 0;
            if has_ai {
                Some(UnitAI::read_from(&mut input, version)?)
            } else {
                None
            }
        };
        attrs.town_bell_flag = input.read_i8()?;
        attrs.town_bell_target_id = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        attrs.town_bell_target_location = {
            let location = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
            if location.0 >= 0.0 {
                Some(location)
            } else {
                None
            }
        };
        attrs.town_bell_target_id_2 = match input.read_i32::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        attrs.town_bell_target_type = input.read_u32::<LE>()?;
        attrs.town_bell_action = input.read_u32::<LE>()?;
        attrs.berserker_timer = input.read_f32::<LE>()?;
        attrs.num_builders = input.read_u8()?;
        attrs.num_healers = input.read_u8()?;
        Ok(attrs)
    }

    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        todo!()
    }
}
