use crate::element::{OptionalReadableElement, ReadableHeaderElement, WritableHeaderElement};
use crate::reader::{Peek, RecordingHeaderReader};
use crate::unit_action::UnitAction;
use crate::unit_type::UnitBaseClass;
use crate::GameVariant::DefinitiveEdition;
use crate::Result;
use crate::{ObjectID, PlayerID};
use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
pub use genie_dat::sprite::SpriteID;
pub use genie_dat::terrain::TerrainID;
pub use genie_dat::unit_type::AttributeCost;
use genie_dat::unit_type::UnitType;
use genie_support::{read_opt_u32, ReadSkipExt, ReadStringsExt};
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
    pub building: Option<BuildingUnitAttributes>,
}

impl OptionalReadableElement for Unit {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Option<Self>> {
        let raw_class = input.read_u8()?;
        if raw_class == 0 {
            return Ok(None);
        }
        let unit_base_class = raw_class.try_into().unwrap();
        let static_ = StaticUnitAttributes::read_from(input)?;
        let mut unit = Self {
            unit_base_class,
            static_,
            animated: None,
            moving: None,
            action: None,
            base_combat: None,
            missile: None,
            combat: None,
            building: None,
        };
        if unit_base_class >= UnitBaseClass::Animated {
            unit.animated = Some(AnimatedUnitAttributes::read_from(input)?);
        }
        if unit_base_class >= UnitBaseClass::Moving {
            unit.moving = Some(MovingUnitAttributes::read_from(input)?);
        }
        if unit_base_class >= UnitBaseClass::Action {
            unit.action = Some(ActionUnitAttributes::read_from(input)?);
        }
        if unit_base_class >= UnitBaseClass::BaseCombat {
            unit.base_combat = Some(BaseCombatUnitAttributes::read_from(input)?);
        }
        if unit_base_class >= UnitBaseClass::Missile {
            unit.missile = Some(MissileUnitAttributes::read_from(input)?);
        }
        if unit_base_class >= UnitBaseClass::Combat {
            unit.combat = Some(CombatUnitAttributes::read_from(input)?);
        }
        if unit_base_class >= UnitBaseClass::Building {
            unit.building = Some(BuildingUnitAttributes::read_from(input)?);
        }

        if unit_base_class == UnitBaseClass::Moving && input.variant() >= DefinitiveEdition {
            input.skip(17)?;
        }

        Ok(Some(unit))
    }
}

impl WritableHeaderElement for Unit {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        let raw_class = self.unit_base_class as u8;
        output.write_u8(raw_class)?;
        self.static_.write_to(output)?;
        if let Some(animated) = &self.animated {
            animated.write_to(output)?;
        }
        if let Some(moving) = &self.moving {
            moving.write_to(output)?;
        }
        if let Some(action) = &self.action {
            action.write_to(output)?;
        }
        if let Some(base_combat) = &self.base_combat {
            base_combat.write_to(output)?;
        }
        if let Some(missile) = &self.missile {
            missile.write_to(output)?;
        }
        if let Some(combat) = &self.combat {
            combat.write_to(output)?;
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

impl ReadableHeaderElement for SpriteNodeAnimation {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
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
}

impl WritableHeaderElement for SpriteNodeAnimation {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
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

impl OptionalReadableElement for SpriteNode {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Option<Self>> {
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
            node.animation = Some(SpriteNodeAnimation::read_from(input)?);
        }
        node.order = input.read_u8()?;
        node.flag = input.read_u8()?;
        node.count = input.read_u8()?;
        Ok(Some(node))
    }
}

impl WritableHeaderElement for SpriteNode {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        let ty = if self.animation.is_some() { 2 } else { 1 };
        output.write_u8(ty)?;
        output.write_u16::<LE>(self.id.into())?;
        output.write_u32::<LE>(self.x)?;
        output.write_u32::<LE>(self.y)?;
        output.write_u16::<LE>(self.frame)?;
        output.write_u8(if self.invisible { 1 } else { 0 })?;
        if let Some(animation) = &self.animation {
            animation.write_to(output)?;
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

impl ReadableHeaderElement for SpriteList {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut sprites = vec![];
        while let Some(node) = SpriteNode::read_from(input)? {
            sprites.push(node);
        }
        Ok(Self { sprites })
    }
}

impl WritableHeaderElement for SpriteList {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        for sprite in &self.sprites {
            sprite.write_to(output)?;
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
    pub de_effect_block: Option<DeEffectBlock>,
}

#[derive(Debug, Default, Clone)]
pub struct DeEffectBlock {
    pub has_effect: bool,
    pub effect_name: Option<String>,
}

impl ReadableHeaderElement for StaticUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.owner_id = input.read_u8()?.into();
        attrs.unit_type_id = input.read_u16::<LE>()?.into();
        attrs.sprite_id = input.read_u16::<LE>()?.into();
        attrs.garrisoned_in_id = read_opt_u32(input)?;
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
        if input.version() < 11.58 {
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
        attrs.group_id = read_opt_u32(input)?;
        attrs.roo_already_called = input.read_u8()?;

        if input.variant() >= DefinitiveEdition {
            input.skip(19)?;
        }

        if input.read_u8()? != 0 {
            attrs.sprite_list = Some(SpriteList::read_from(input)?);
        }

        if input.variant() >= DefinitiveEdition {
            input.skip(4)?;
            let has_effect = input.read_u8()? == 1;

            let effect_name = if has_effect {
                input.skip(1)?;

                let effect_name = input.read_hd_style_str()?;
                if effect_name.is_some() {
                    // effect arguments?
                    input.skip(34)?;
                }

                effect_name
            } else {
                input.skip(1)?;
                None
            };

            input.skip(4)?;

            if input.version() >= 13.15 {
                input.skip(5)?;
            }

            if input.version() >= 13.17 {
                input.skip(2)?;
            }

            if input.version() >= 13.34 {
                input.skip(12)?;
            }

            attrs.de_effect_block = Some(DeEffectBlock {
                has_effect,
                effect_name,
            });
        }

        Ok(attrs)
    }
}

impl WritableHeaderElement for StaticUnitAttributes {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
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

impl ReadableHeaderElement for AnimatedUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let speed = input.read_f32::<LE>()?;
        Ok(Self { speed })
    }
}

impl WritableHeaderElement for AnimatedUnitAttributes {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
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

impl ReadableHeaderElement for PathData {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut path = Self::default();
        path.id = input.read_u32::<LE>()?;
        path.linked_path_type = input.read_u32::<LE>()?;
        path.waypoint_level = input.read_u32::<LE>()?;
        path.path_id = input.read_u32::<LE>()?;
        path.waypoint = input.read_u32::<LE>()?;
        if input.version() < 10.25 {
            path.disable_flags = Some(input.read_u32::<LE>()?);
            if input.version() >= 10.20 {
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
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MovementData {
    pub velocity: (f32, f32, f32),
    pub acceleration: (f32, f32, f32),
}

impl MovementData {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
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

impl ReadableHeaderElement for MovingUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
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
        attrs.current_terrain_exception = (read_opt_u32(input)?, read_opt_u32(input)?);
        attrs.waiting_to_move = input.read_u8()?;
        attrs.wait_delays_count = input.read_u8()?;
        attrs.on_ground = input.read_u8()?;
        attrs.path_data = {
            let num_paths = input.read_u32::<LE>()?;
            let mut paths = vec![];
            for _ in 0..num_paths {
                paths.push(PathData::read_from(input)?);
            }
            paths
        };
        if input.read_u32::<LE>()? != 0 {
            attrs.future_path_data = Some(PathData::read_from(input)?);
        }
        if input.read_u32::<LE>()? != 0 {
            attrs.movement_data = Some(MovementData::read_from(input)?);
        }

        if input.variant() >= DefinitiveEdition && input.version() < 13.2 {
            input.skip(2)?;
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
}

impl WritableHeaderElement for MovingUnitAttributes {}

#[derive(Debug, Default, Clone)]
pub struct ActionUnitAttributes {
    pub waiting: bool,
    pub command_flag: u8,
    pub selected_group_info: u16,
    pub actions: Vec<UnitAction>,
}

impl ReadableHeaderElement for ActionUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.waiting = input.read_u8()? != 0;
        if input.version() >= 6.5 {
            attrs.command_flag = input.read_u8()?;
        }
        if input.version() >= 11.58 {
            attrs.selected_group_info = input.read_u16::<LE>()?;
        }
        attrs.actions = UnitAction::read_list_from(input)?;
        Ok(attrs)
    }
}

impl WritableHeaderElement for ActionUnitAttributes {}

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

impl ReadableHeaderElement for BaseCombatUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut attrs = Self::default();
        if input.version() >= 9.05 {
            attrs.formation_id = input.read_u8()?;
            attrs.formation_row = input.read_u8()?;
            attrs.formation_column = input.read_u8()?;
        }
        attrs.attack_timer = input.read_f32::<LE>()?;
        if input.version() >= 2.01 {
            attrs.capture_flag = input.read_u8()?;
        }
        if input.version() >= 9.09 {
            attrs.multi_unified_points = input.read_u8()?;
            attrs.large_object_radius = input.read_u8()?;
        }
        if input.version() >= 10.02 {
            attrs.attack_count = input.read_u32::<LE>()?;
        }
        Ok(attrs)
    }
}

impl WritableHeaderElement for BaseCombatUnitAttributes {}

#[derive(Debug, Default, Clone)]
pub struct MissileUnitAttributes {
    pub max_range: f32,
    pub fired_from_id: ObjectID,
    pub own_base: Option<UnitType>,
}

impl ReadableHeaderElement for MissileUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.max_range = input.read_f32::<LE>()?;
        attrs.fired_from_id = input.read_u32::<LE>()?.into();
        attrs.own_base = {
            if input.read_u8()? == 0 {
                None
            } else {
                let version = input.version();
                Some(UnitType::read_from(&mut *input, version)?)
            }
        };
        Ok(attrs)
    }
}

impl WritableHeaderElement for MissileUnitAttributes {}

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

impl ReadableHeaderElement for UnitAIOrder {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
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
}

#[derive(Debug, Default, Clone)]
pub struct UnitAINotification {
    pub caller: u32,
    pub recipient: u32,
    pub notification_type: u32,
    pub params: (u32, u32, u32),
}

impl ReadableHeaderElement for UnitAINotification {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
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
}

#[derive(Debug, Default, Clone)]
pub struct UnitAIOrderHistory {
    order: u32,
    action: u32,
    time: u32,
    position: (f32, f32, f32),
    target_id: ObjectID,
    target_attack_category: Option<u32>,
    target_position: (f32, f32, f32),
}

impl ReadableHeaderElement for UnitAIOrderHistory {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
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
        if input.version() >= 10.50 {
            order.target_attack_category = read_opt_u32(input)?;
        }
        order.target_position = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        Ok(order)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UnitAIRetargetEntry {
    pub target_id: ObjectID,
    pub retarget_timeout: u32,
}

impl ReadableHeaderElement for UnitAIRetargetEntry {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let target_id = input.read_u32::<LE>()?.into();
        let retarget_timeout = input.read_u32::<LE>()?;
        Ok(Self {
            target_id,
            retarget_timeout,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct Waypoint {
    pub location: (f32, f32, f32),
    pub facet_to_next_waypoint: u8,
}

impl ReadableHeaderElement for Waypoint {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
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
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PatrolPath {}

impl ReadableHeaderElement for PatrolPath {
    fn read_from<R: Read>(_: &mut RecordingHeaderReader<R>) -> Result<Self> {
        unimplemented!()
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

impl ReadableHeaderElement for UnitAI {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut ai = Self::default();
        ai.mood = read_opt_u32(input)?;
        ai.current_order = read_opt_u32(input)?;
        ai.current_order_priority = read_opt_u32(input)?;
        ai.current_action = read_opt_u32(input)?;
        ai.current_target = read_opt_u32(input)?;
        ai.current_target_type = match input.read_u16::<LE>()? {
            0xFFFF => None,
            id => Some(id.try_into().unwrap()),
        };
        input.skip(2)?;
        ai.current_target_location = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        ai.desired_target_distance = input.read_f32::<LE>()?;
        ai.last_action = read_opt_u32(input)?;
        ai.last_order = read_opt_u32(input)?;
        ai.last_target = read_opt_u32(input)?;
        ai.last_target_type = read_opt_u32(input)?;
        ai.last_update_type = read_opt_u32(input)?;
        ai.idle_timer = input.read_u32::<LE>()?;
        ai.idle_timeout = input.read_u32::<LE>()?;
        ai.adjusted_idle_timeout = input.read_u32::<LE>()?;
        ai.secondary_timer = input.read_u32::<LE>()?;
        ai.lookaround_timer = input.read_u32::<LE>()?;
        ai.lookaround_timeout = input.read_u32::<LE>()?;
        ai.defend_target = read_opt_u32(input)?;
        ai.defense_buffer = input.read_f32::<LE>()?;
        ai.last_world_position = Waypoint::read_from(input)?;

        if input.version() >= 20.06 {
            input.skip(8)?;
        }

        ai.orders = {
            let num_orders = input.read_u32::<LE>()?;
            let mut orders = vec![];
            for _ in 0..num_orders {
                orders.push(UnitAIOrder::read_from(input)?);
            }
            orders
        };
        ai.notifications = {
            let num_notifications = input.read_u32::<LE>()?;
            let mut notifications = vec![];
            for _ in 0..num_notifications {
                notifications.push(UnitAINotification::read_from(input)?);
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
        ai.alert_mode_object_id = read_opt_u32(input)?;
        ai.patrol_path = {
            let has_path = input.read_u32::<LE>()? != 0;
            if has_path {
                Some(PatrolPath::read_from(input)?)
            } else {
                None
            }
        };
        ai.patrol_current_waypoint = input.read_u32::<LE>()?;
        if input.version() >= 10.48 {
            ai.order_history = {
                let num_orders = input.read_u32::<LE>()?;
                let mut orders = vec![];
                for _ in 0..num_orders {
                    orders.push(UnitAIOrderHistory::read_from(input)?);
                }
                orders
            };
        }
        if input.version() >= 10.50 {
            ai.last_retarget_time = input.read_u32::<LE>()?;
        }
        if input.version() >= 11.04 {
            ai.randomized_retarget_timer = input.read_u32::<LE>()?;
        }
        if input.version() >= 11.05 {
            ai.retarget_entries = {
                let num_entries = input.read_u32::<LE>()?;
                let mut entries = vec![];
                for _ in 0..num_entries {
                    entries.push(UnitAIRetargetEntry::read_from(input)?);
                }
                entries
            };
        }
        if input.version() >= 11.14 {
            ai.best_unit_to_attack = read_opt_u32(input)?;
        }
        if input.version() >= 11.44 {
            ai.formation_type = input.read_u8()?;
        }

        if input.variant() >= DefinitiveEdition {
            input.skip(4)?;
        }

        Ok(ai)
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

impl ReadableHeaderElement for CombatUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut attrs = Self::default();

        if input.variant() >= DefinitiveEdition {
            input.skip(18)?;
        }

        attrs.next_volley = input.read_u8()?;
        attrs.using_special_attack_animation = input.read_u8()?;
        attrs.own_base = {
            if input.read_u8()? == 0 {
                None
            } else {
                let version = input.version();
                Some(UnitType::read_from(&mut *input, version)?)
            }
        };
        for amount in attrs.attribute_amounts.iter_mut() {
            *amount = input.read_u16::<LE>()?;
        }
        if input.version() >= 9.16 {
            attrs.decay_timer = input.read_u16::<LE>()?;
        }
        if input.version() >= 9.61 {
            attrs.raider_build_countdown = input.read_u32::<LE>()?;
        }
        if input.version() >= 9.65 {
            attrs.locked_down_count = input.read_u32::<LE>()?;
        }
        if input.version() >= 11.56 {
            attrs.inside_garrison_count = input.read_u8()?;
        }
        attrs.unit_ai = {
            let has_ai = input.read_u32::<LE>()? != 0;
            if has_ai {
                Some(UnitAI::read_from(input)?)
            } else {
                None
            }
        };

        // https://github.com/happyleavesaoc/aoc-mgz/blob/ce4e5dc6184fcd005d0c50d3abac58dd863778be/mgz/header/objects.py#L361
        // ???
        if input.peek(5)? != b"\x00\xff\xff\xff\xff" {
            input.skip(13)?;
        }

        if input.version() >= 10.30 {
            attrs.town_bell_flag = input.read_i8()?;
            attrs.town_bell_target_id = read_opt_u32(input)?;
            attrs.town_bell_target_location = {
                let location = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
                if location.0 >= 0.0 {
                    Some(location)
                } else {
                    None
                }
            };
        }
        if input.version() >= 11.71 {
            attrs.town_bell_target_id_2 = read_opt_u32(input)?;
            attrs.town_bell_target_type = input.read_u32::<LE>()?;
        }
        if input.version() >= 11.74 {
            attrs.town_bell_action = input.read_u32::<LE>()?;
        }
        if input.version() >= 10.42 {
            attrs.berserker_timer = input.read_f32::<LE>()?;
        }
        if input.version() >= 10.46 {
            attrs.num_builders = input.read_u8()?;
        }
        if input.version() >= 11.69 {
            attrs.num_healers = input.read_u8()?;
        }

        if input.version() >= 20.06 {
            input.skip(4)?;
        }

        Ok(attrs)
    }
}

impl WritableHeaderElement for CombatUnitAttributes {}

#[derive(Debug, Clone)]
pub enum GatherPoint {
    Location { x: f32, y: f32, z: f32 },
    Object { id: ObjectID, unit_type: UnitTypeID },
}

#[derive(Debug, Default, Clone)]
pub struct ProductionQueueEntry {
    pub unit_type_id: UnitTypeID,
    pub count: u16,
}

impl ReadableHeaderElement for ProductionQueueEntry {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let unit_type_id = input.read_u16::<LE>()?.into();
        let count = input.read_u16::<LE>()?;
        Ok(Self {
            unit_type_id,
            count,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct BuildingUnitAttributes {
    /// Is this building fully built?
    pub built: bool,
    /// Number of build points: how much work villagers have to do to build this building.
    pub build_points: f32,
    /// The build item ID for this building. Only used by AIs.
    pub unique_build_id: Option<u32>,
    /// The culture / architecture graphics set to use for this building.
    pub culture: u8,
    /// Is this building on fire?
    pub burning: u8,
    pub last_burn_time: u32,
    pub last_garrison_time: u32,
    /// The number of relics currently stored inside this building.
    pub relic_count: u32,
    /// The number of "specific relics" currently stored inside this building(?).
    ///
    /// This specific relic count generates 2× as much gold as normal relics, but appears to be
    /// otherwise unused.
    pub specific_relic_count: u32,
    /// Gather point for units trained from this building.
    pub gather_point: Option<GatherPoint>,
    pub desolid_flag: bool,
    pub pending_order: u32,
    /// The "owner" building, if this building object is part of a larger building like a Town
    /// Center.
    pub linked_owner: Option<ObjectID>,
    /// The IDs of the children of this building object, also known as "annex buildings".
    pub linked_children: ArrayVec<ObjectID, 4>,
    pub captured_unit_count: u8,
    pub extra_actions: Vec<UnitAction>,
    pub research_actions: Vec<UnitAction>,
    /// The current active production queue.
    pub production_queue: Vec<ProductionQueueEntry>,
    /// Cumulative count of queued units.
    pub production_queue_total_units: u16,
    pub production_queue_enabled: bool,
    /// The actions currently in the production queue.
    pub production_queue_actions: Vec<UnitAction>,
    pub endpoint: (f32, f32, f32),
    pub gate_locked: u32,
    pub first_update: u32,
    pub close_timer: u32,
    pub terrain_type: Option<TerrainID>,
    pub semi_asleep: bool,
    /// Should this building be rendered with the snow graphic?
    pub snow_flag: bool,
}

impl ReadableHeaderElement for BuildingUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.built = input.read_u8()? != 0;
        attrs.build_points = input.read_f32::<LE>()?;
        attrs.unique_build_id = read_opt_u32(input)?;
        attrs.culture = input.read_u8()?;
        attrs.burning = input.read_u8()?;
        attrs.last_burn_time = input.read_u32::<LE>()?;
        attrs.last_garrison_time = input.read_u32::<LE>()?;
        attrs.relic_count = input.read_u32::<LE>()?;
        attrs.specific_relic_count = input.read_u32::<LE>()?;
        attrs.gather_point = {
            let exists = input.read_u32::<LE>()? != 0;
            let location = GatherPoint::Location {
                x: input.read_f32::<LE>()?,
                y: input.read_f32::<LE>()?,
                z: input.read_f32::<LE>()?,
            };
            let object_id = input.read_i32::<LE>()?;
            let unit_type_id = input.read_i16::<LE>()?;
            match (exists, object_id, unit_type_id) {
                (false, _, _) => None,
                (true, -1, -1) => Some(location),
                (true, id, unit_type_id) => Some(GatherPoint::Object {
                    id: id.try_into().unwrap(),
                    unit_type: unit_type_id.try_into().unwrap(),
                }),
            }
        };
        attrs.desolid_flag = input.read_u8()? != 0;
        if input.version() >= 10.54 {
            attrs.pending_order = input.read_u32::<LE>()?;
        }
        attrs.linked_owner = read_opt_u32(input)?;
        attrs.linked_children = {
            let mut children: ArrayVec<ObjectID, 4> = Default::default();

            let num_children = if input.variant() >= DefinitiveEdition {
                3
            } else {
                4
            };

            for _ in 0..num_children {
                let id = input.read_i32::<LE>()?;
                if id != -1 {
                    children.push(id.try_into().unwrap());
                }
            }
            children
        };
        attrs.captured_unit_count = input.read_u8()?;
        attrs.extra_actions = UnitAction::read_list_from(input)?;

        if input.variant() != DefinitiveEdition {
            attrs.research_actions = UnitAction::read_list_from(input)?;
        }

        attrs.production_queue = {
            let capacity = input.read_u16::<LE>()?;
            let mut queue = vec![ProductionQueueEntry::default(); capacity as usize];
            for entry in queue.iter_mut() {
                *entry = ProductionQueueEntry::read_from(input)?;
            }
            let _size = input.read_u16::<LE>()?;
            queue
        };
        attrs.production_queue_total_units = input.read_u16::<LE>()?;
        attrs.production_queue_enabled = input.read_u8()? != 0;
        attrs.production_queue_actions = UnitAction::read_list_from(input)?;
        if input.version() >= 10.65 {
            // game reads into the same value twice, while there are two separate fields of this
            // type. likely a bug, but it doesn't appear to cause issues? is this unused?
            attrs.endpoint = (
                input.read_f32::<LE>()?,
                input.read_f32::<LE>()?,
                input.read_f32::<LE>()?,
            );
            attrs.endpoint = (
                input.read_f32::<LE>()?,
                input.read_f32::<LE>()?,
                input.read_f32::<LE>()?,
            );
            attrs.gate_locked = input.read_u32::<LE>()?;
            attrs.first_update = input.read_u32::<LE>()?;
            attrs.close_timer = input.read_u32::<LE>()?;
        }
        if input.version() >= 10.67 {
            attrs.terrain_type = Some(input.read_u8()?.into());
        }
        if input.version() >= 11.43 {
            attrs.semi_asleep = input.read_u8()? != 0;
        }
        if input.version() >= 11.54 {
            attrs.snow_flag = input.read_u8()? != 0;
        }

        if input.variant() >= DefinitiveEdition {
            input.skip(1)?;
        }

        Ok(attrs)
    }
}
