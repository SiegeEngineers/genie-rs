use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::{
    convert::TryInto,
    io::{self, Read, Result, Write},
};
use crate::{
    task::TaskList,
    sprite::{GraphicID, SpriteID},
    sound::SoundID,
};

macro_rules! fallible_try_into {
    ($from:ident, $to:ty) => {
        impl std::convert::TryFrom<$from> for $to {
            type Error = std::num::TryFromIntError;
            fn try_from(n: $from) -> std::result::Result<Self, Self::Error> {
                n.0.try_into()
            }
        }
    }
}

/// An ID identifying a unit type.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UnitTypeID(u16);

impl From<u16> for UnitTypeID {
    fn from(n: u16) -> Self {
        UnitTypeID(n)
    }
}

impl From<UnitTypeID> for u16 {
    fn from(n: UnitTypeID) -> Self {
        n.0
    }
}

fallible_try_into!(UnitTypeID, i16);

/// An ID identifying a string resource.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StringID(u32);

impl From<u16> for StringID {
    fn from(n: u16) -> Self {
        StringID(n.into())
    }
}

impl From<u32> for StringID {
    fn from(n: u32) -> Self {
        StringID(n)
    }
}

fallible_try_into!(StringID, u16);
fallible_try_into!(StringID, i16);

impl From<StringID> for u32 {
    fn from(n: StringID) -> Self {
        n.0
    }
}

pub type UnitClass = u16;

/// Helper trait to map a Result<T> to Result<From<T>>
trait MapInto<T> {
    fn map_into(self) -> T;
}
impl<Source, Target> MapInto<Result<Target>> for Result<Source>
    where Target: From<Source>
{
    fn map_into(self) -> Result<Target> {
        self.map(|v| v.into())
    }
}
impl<Source, Target> MapInto<Option<Target>> for Option<Source>
    where Target: From<Source>
{
    fn map_into(self) -> Option<Target> {
        self.map(|v| v.into())
    }
}

#[derive(Debug, Clone)]
pub enum UnitType {
    Static(StaticUnitType),
    Tree(TreeUnitType),
    Animated(AnimatedUnitType),
    Doppleganger(DopplegangerUnitType),
    Moving(MovingUnitType),
    Action(ActionUnitType),
    BaseCombat(BaseCombatUnitType),
    Missile(MissileUnitType),
    Combat(CombatUnitType),
    Building(BuildingUnitType),
}

macro_rules! cast_unit_type {
    ($struct:ident, $tag:ident) => {
        impl From<$struct> for UnitType {
            fn from(v: $struct) -> Self {
                UnitType::$tag(v)
            }
        }
    };
}

cast_unit_type!(StaticUnitType, Static);
cast_unit_type!(TreeUnitType, Tree);
cast_unit_type!(AnimatedUnitType, Animated);
cast_unit_type!(DopplegangerUnitType, Doppleganger);
cast_unit_type!(MovingUnitType, Moving);
cast_unit_type!(ActionUnitType, Action);
cast_unit_type!(BaseCombatUnitType, BaseCombat);
cast_unit_type!(MissileUnitType, Missile);
cast_unit_type!(CombatUnitType, Combat);
cast_unit_type!(BuildingUnitType, Building);

impl UnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let unit_type = input.read_u8()?;
        match unit_type {
            10 => StaticUnitType::from(input).map_into(),
            15 => TreeUnitType::from(input).map_into(),
            20 => AnimatedUnitType::from(input).map_into(),
            25 => DopplegangerUnitType::from(input).map_into(),
            30 => MovingUnitType::from(input).map_into(),
            40 => ActionUnitType::from(input).map_into(),
            50 => BaseCombatUnitType::from(input).map_into(),
            60 => MissileUnitType::from(input).map_into(),
            70 => CombatUnitType::from(input).map_into(),
            80 => BuildingUnitType::from(input).map_into(),
            _ => panic!("unexpected unit type {}, this is probably a bug", unit_type),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UnitAttribute {
    attribute_type: u16,
    amount: f32,
    flag: u8,
}

impl UnitAttribute {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            attribute_type: input.read_u16::<LE>()?,
            amount: input.read_f32::<LE>()?,
            flag: input.read_u8()?,
        })
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u16::<LE>(self.attribute_type)?;
        output.write_f32::<LE>(self.amount)?;
        output.write_u8(self.flag)?;
        Ok(())
    }
}


#[derive(Debug, Default, Clone)]
pub struct DamageSprite {
    sprite: SpriteID,
    damage_percent: u16,
    flag: u8,
}

impl DamageSprite {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            sprite: input.read_u16::<LE>()?.into(),
            damage_percent: input.read_u16::<LE>()?,
            flag: input.read_u8()?,
        })
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u16::<LE>(self.sprite.into())?;
        output.write_u16::<LE>(self.damage_percent)?;
        output.write_u8(self.flag)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct StaticUnitType {
    name: String,
    id: UnitTypeID,
    string_id: StringID,
    string_id2: Option<StringID>,
    unit_class: UnitClass,
    standing_sprite_1: Option<SpriteID>,
    standing_sprite_2: Option<SpriteID>,
    dying_sprite: Option<SpriteID>,
    undead_sprite: Option<SpriteID>,
    undead_flag: u8,
    hp: u16,
    los: f32,
    garrison_capacity: u8,
    radius: (f32, f32, f32),
    train_sound: Option<SoundID>,
    damage_sound: Option<SoundID>,
    death_spawn: Option<UnitTypeID>,
    sort_number: u8,
    can_be_built_on: bool,
    button_picture: Option<GraphicID>,
    hide_in_scenario_editor: bool,
    portrait_picture: Option<GraphicID>,
    enabled: bool,
    disabled: bool,
    tile_req: (i16, i16),
    center_tile_req: (i16, i16),
    construction_radius: (f32, f32),
    elevation_flag: bool,
    fog_flag: bool,
    terrain_restriction_id: u16,
    movement_type: u8,
    attribute_max_amount: u16,
    attribute_rot: f32,
    area_effect_level: u8,
    combat_level: u8,
    select_level: u8,
    map_draw_level: u8,
    unit_level: u8,
    multiple_attribute_mod: f32,
    map_color: u8,
    help_string_id: StringID,
    help_page_id: u32,
    hotkey_id: u32,
    recyclable: bool,
    track_as_resource: bool,
    create_doppleganger: bool,
    resource_group: u8,
    occlusion_mask: u8,
    obstruction_type: u8,
    selection_shape: u8,
    object_flags: u32,
    civilization: u8,
    attribute_piece: u8,
    outline_radius: (f32, f32, f32),
    attributes: [UnitAttribute; 3],
    damage_sprites: Vec<DamageSprite>,
    selected_sound: Option<SoundID>,
    death_sound: Option<SoundID>,
    attack_reaction: u8,
    convert_terrain_flag: u8,
    copy_id: u16,
    unit_group: u16,
}

fn read_opt_u16<R: Read>(input: &mut R) -> Result<Option<u16>> {
    let v = input.read_i16::<LE>()?;
    if v == -1 { return Ok(None); }
    Ok(Some(v.try_into().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?))
}

impl StaticUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut unit_type = Self::default();
        let name_len = input.read_u16::<LE>()?;
        unit_type.id = input.read_u16::<LE>()?.into();
        unit_type.string_id = input.read_u16::<LE>()?.into();
        unit_type.string_id2 = read_opt_u16(input)?.map_into();
        unit_type.unit_class = input.read_u16::<LE>()?;
        unit_type.standing_sprite_1 = read_opt_u16(input)?.map_into();
        unit_type.standing_sprite_2 = read_opt_u16(input)?.map_into();
        unit_type.dying_sprite = read_opt_u16(input)?.map_into();
        unit_type.undead_sprite = read_opt_u16(input)?.map_into();
        unit_type.undead_flag = input.read_u8()?;
        unit_type.hp = input.read_u16::<LE>()?;
        unit_type.los = input.read_f32::<LE>()?;
        unit_type.garrison_capacity = input.read_u8()?;
        unit_type.radius = (input.read_f32::<LE>()?, input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        unit_type.train_sound = read_opt_u16(input)?.map_into();
        unit_type.damage_sound = read_opt_u16(input)?.map_into();
        unit_type.death_spawn = read_opt_u16(input)?.map_into();
        unit_type.sort_number = input.read_u8()?;
        unit_type.can_be_built_on = input.read_u8()? != 0;
        unit_type.button_picture = read_opt_u16(input)?.map_into();
        unit_type.hide_in_scenario_editor = input.read_u8()? != 0;
        unit_type.portrait_picture = read_opt_u16(input)?.map_into();
        unit_type.enabled = input.read_u8()? != 0;
        unit_type.disabled = input.read_u8()? != 0;
        unit_type.tile_req = (input.read_i16::<LE>()?, input.read_i16::<LE>()?);
        unit_type.center_tile_req = (input.read_i16::<LE>()?, input.read_i16::<LE>()?);
        unit_type.construction_radius = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        unit_type.elevation_flag = input.read_u8()? != 0;
        unit_type.fog_flag = input.read_u8()? != 0;
        unit_type.terrain_restriction_id = input.read_u16::<LE>()?;
        unit_type.movement_type = input.read_u8()?;
        unit_type.attribute_max_amount = input.read_u16::<LE>()?;
        unit_type.attribute_rot = input.read_f32::<LE>()?;
        unit_type.area_effect_level = input.read_u8()?;
        unit_type.combat_level = input.read_u8()?;
        unit_type.select_level = input.read_u8()?;
        unit_type.map_draw_level = input.read_u8()?;
        unit_type.unit_level = input.read_u8()?;
        unit_type.multiple_attribute_mod = input.read_f32::<LE>()?;
        unit_type.map_color = input.read_u8()?;
        unit_type.help_string_id = input.read_u32::<LE>()?.into();
        unit_type.help_page_id = input.read_u32::<LE>()?;
        unit_type.hotkey_id = input.read_u32::<LE>()?;
        unit_type.recyclable = input.read_u8()? != 0;
        unit_type.track_as_resource = input.read_u8()? != 0;
        unit_type.create_doppleganger = input.read_u8()? != 0;
        unit_type.resource_group = input.read_u8()?;
        unit_type.occlusion_mask = input.read_u8()?;
        unit_type.obstruction_type = input.read_u8()?;
        unit_type.selection_shape = input.read_u8()?;
        unit_type.object_flags = input.read_u32::<LE>()?;
        unit_type.civilization = input.read_u8()?;
        unit_type.attribute_piece = input.read_u8()?;
        unit_type.outline_radius = (input.read_f32::<LE>()?, input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        for attr in unit_type.attributes.iter_mut() {
            *attr = UnitAttribute::from(input)?;
        }
        unit_type.damage_sprites = {
            let num_damage_sprites = input.read_u8()?;
            let mut damage_sprites = vec![];
            for _ in 0..num_damage_sprites {
                damage_sprites.push(DamageSprite::from(input)?);
            }
            damage_sprites
        };
        unit_type.selected_sound = read_opt_u16(input)?.map_into();
        unit_type.death_sound = read_opt_u16(input)?.map_into();
        unit_type.attack_reaction = input.read_u8()?;
        unit_type.convert_terrain_flag = input.read_u8()?;
        unit_type.name = {
            let mut bytes = vec![0; usize::from(name_len)];
            input.read_exact(&mut bytes)?;
            String::from_utf8(bytes.iter().cloned().take_while(|b| *b != 0).collect()).unwrap()
        };
        unit_type.copy_id = input.read_u16::<LE>()?;
        unit_type.unit_group = input.read_u16::<LE>()?;
        Ok(unit_type)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u16::<LE>(self.id.into())?;
        output.write_u16::<LE>(self.string_id.try_into().unwrap())?;
        output.write_i16::<LE>(self.string_id2.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_u16::<LE>(self.unit_class)?;
        output.write_i16::<LE>(self.standing_sprite_1.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_i16::<LE>(self.standing_sprite_2.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_i16::<LE>(self.dying_sprite.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_i16::<LE>(self.undead_sprite.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_u8(self.undead_flag)?;
        output.write_u16::<LE>(self.hp)?;
        output.write_f32::<LE>(self.los)?;
        output.write_u8(self.garrison_capacity)?;
        output.write_f32::<LE>(self.radius.0)?;
        output.write_f32::<LE>(self.radius.1)?;
        output.write_f32::<LE>(self.radius.2)?;
        output.write_i16::<LE>(self.train_sound.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_i16::<LE>(self.damage_sound.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_i16::<LE>(self.death_spawn.map(|id| id.0.try_into().unwrap()).unwrap_or(-1))?;
        output.write_u8(self.sort_number)?;
        output.write_u8(if self.can_be_built_on { 1 } else { 0 })?;
        output.write_i16::<LE>(self.button_picture.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_u8(if self.hide_in_scenario_editor { 1 } else { 0 })?;
        output.write_i16::<LE>(self.portrait_picture.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_u8(if self.enabled { 1 } else { 0 })?;
        output.write_u8(if self.disabled { 1 } else { 0 })?;
        output.write_i16::<LE>(self.tile_req.0)?;
        output.write_i16::<LE>(self.tile_req.1)?;
        output.write_i16::<LE>(self.center_tile_req.0)?;
        output.write_i16::<LE>(self.center_tile_req.1)?;
        output.write_f32::<LE>(self.construction_radius.0)?;
        output.write_f32::<LE>(self.construction_radius.1)?;
        output.write_u8(if self.elevation_flag { 1 } else { 0 })?;
        output.write_u8(if self.fog_flag { 1 } else { 0 })?;
        output.write_u16::<LE>(self.terrain_restriction_id)?;
        output.write_u8(self.movement_type)?;
        output.write_u16::<LE>(self.attribute_max_amount)?;
        output.write_f32::<LE>(self.attribute_rot)?;
        output.write_u8(self.area_effect_level)?;
        output.write_u8(self.combat_level)?;
        output.write_u8(self.select_level)?;
        output.write_u8(self.map_draw_level)?;
        output.write_u8(self.unit_level)?;
        output.write_f32::<LE>(self.multiple_attribute_mod)?;
        output.write_u8(self.map_color)?;
        output.write_u32::<LE>(self.help_string_id.try_into().unwrap())?;
        output.write_u32::<LE>(self.help_page_id)?;
        output.write_u32::<LE>(self.hotkey_id)?;
        output.write_u8(if self.recyclable { 1 } else { 0 })?;
        output.write_u8(if self.track_as_resource { 1 } else { 0 })?;
        output.write_u8(if self.create_doppleganger { 1 } else { 0 })?;
        output.write_u8(self.resource_group)?;
        output.write_u8(self.occlusion_mask)?;
        output.write_u8(self.obstruction_type)?;
        output.write_u8(self.selection_shape)?;
        output.write_u32::<LE>(self.object_flags)?;
        output.write_u8(self.civilization)?;
        output.write_u8(self.attribute_piece)?;
        output.write_f32::<LE>(self.outline_radius.0)?;
        output.write_f32::<LE>(self.outline_radius.1)?;
        output.write_f32::<LE>(self.outline_radius.2)?;
        for attr in &self.attributes {
            attr.write_to(output)?;
        }
        output.write_u8(self.damage_sprites.len().try_into().unwrap())?;
        for sprite in &self.damage_sprites {
            sprite.write_to(output)?;
        }
        output.write_i16::<LE>(self.selected_sound.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_i16::<LE>(self.death_sound.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_u8(self.attack_reaction)?;
        output.write_u8(self.convert_terrain_flag)?;
        output.write_u16::<LE>(self.copy_id)?;
        output.write_u16::<LE>(self.unit_group)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct TreeUnitType(StaticUnitType);

impl TreeUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        StaticUnitType::from(input).map(Self)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        self.0.write_to(output)
    }
}


#[derive(Debug, Default, Clone)]
pub struct AnimatedUnitType {
    superclass: StaticUnitType,
    speed: f32,
}

impl AnimatedUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            superclass: StaticUnitType::from(input)?,
            speed: input.read_f32::<LE>()?,
        })
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        self.superclass.write_to(output)?;
        output.write_f32::<LE>(self.speed)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct DopplegangerUnitType(AnimatedUnitType);

impl DopplegangerUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        AnimatedUnitType::from(input).map(Self)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        self.0.write_to(output)
    }
}

#[derive(Debug, Default, Clone)]
pub struct MovingUnitType {
    superclass: AnimatedUnitType,
    move_sprite: SpriteID,
    run_sprite: SpriteID,
    turn_speed: f32,
    size_class: u8,
    trailing_unit: UnitTypeID,
    trailing_options: u8,
    trailing_spacing: f32,
    move_algorithm: u8,
    turn_radius: f32,
    turn_radius_speed: f32,
    maximum_yaw_per_second_moving: f32,
    stationary_yaw_revolution_time: f32,
    maximum_yaw_per_second_stationary: f32,
}

impl MovingUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut unit_type = Self {
            superclass: AnimatedUnitType::from(input)?,
            ..Default::default()
        };
        Ok(unit_type)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ActionUnitType {
    superclass: MovingUnitType,
    default_task: u16,
    search_radius: f32,
    work_rate: f32,
    drop_site: UnitTypeID,
    backup_drop_site: UnitTypeID,
    task_by_group: u8,
    command_sound: Option<SoundID>,
    move_sound: Option<SoundID>,
    /// Task list for older versions; newer game versions store the task list at the root of the
    /// dat file, and use `unit_type.copy_id` to refer to one of those task lists.
    tasks: Option<TaskList>,
}

impl ActionUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut unit_type = Self {
            superclass: MovingUnitType::from(input)?,
            ..Default::default()
        };
        Ok(unit_type)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WeaponInfo {
    weapon_type: i16,
    value: i16,
}

#[derive(Debug, Default, Clone)]
pub struct BaseCombatUnitType {
    superclass: ActionUnitType,
    base_armor: u16,
    weapons: Vec<WeaponInfo>,
    armors: Vec<WeaponInfo>,
}

impl BaseCombatUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut unit_type = Self {
            superclass: ActionUnitType::from(input)?,
            ..Default::default()
        };
        Ok(unit_type)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct MissileUnitType {
    superclass: BaseCombatUnitType,
    missile_type: u8,
    targetting_type: u8,
    missile_hit_info: u8,
    missile_die_info: u8,
    area_effect_specials: u8,
    ballistics_ratio: f32,
}

impl MissileUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut unit_type = Self {
            superclass: BaseCombatUnitType::from(input)?,
            ..Default::default()
        };
        Ok(unit_type)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct CombatUnitType {
    superclass: BaseCombatUnitType,
    build_pts_required: u16,
}

impl CombatUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut unit_type = Self {
            superclass: BaseCombatUnitType::from(input)?,
            ..Default::default()
        };
        Ok(unit_type)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct BuildingUnitType {
    superclass: CombatUnitType,
}

impl BuildingUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut unit_type = Self {
            superclass: CombatUnitType::from(input)?,
            ..Default::default()
        };
        dbg!(&unit_type);
        Ok(unit_type)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        Ok(())
    }
}
