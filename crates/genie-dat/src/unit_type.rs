use crate::{
    sound::SoundID,
    sprite::{GraphicID, SpriteID},
    task::TaskList,
    terrain::TerrainID,
    GameVersion,
};
use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{
    fallible_try_from, fallible_try_into, infallible_try_into, read_opt_u16, MapInto,
};
use std::{
    convert::TryInto,
    io::{Read, Result, Write},
};

pub type TechID = u16;

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

impl From<UnitTypeID> for usize {
    fn from(n: UnitTypeID) -> Self {
        n.0.into()
    }
}

fallible_try_into!(UnitTypeID, i16);
infallible_try_into!(UnitTypeID, u32);
fallible_try_from!(UnitTypeID, i32);
fallible_try_from!(UnitTypeID, u32);

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
fallible_try_from!(StringID, i32);

impl From<StringID> for u32 {
    fn from(n: StringID) -> Self {
        n.0
    }
}

pub type UnitClass = u16;

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
    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let unit_type = input.read_u8()?;
        match unit_type {
            10 => StaticUnitType::from(input, version).map_into(),
            15 => TreeUnitType::from(input, version).map_into(),
            20 => AnimatedUnitType::from(input, version).map_into(),
            25 => DopplegangerUnitType::from(input, version).map_into(),
            30 => MovingUnitType::from(input, version).map_into(),
            40 => ActionUnitType::from(input, version).map_into(),
            50 => BaseCombatUnitType::from(input, version).map_into(),
            60 => MissileUnitType::from(input, version).map_into(),
            70 => CombatUnitType::from(input, version).map_into(),
            80 => BuildingUnitType::from(input, version).map_into(),
            _ => panic!("unexpected unit type {}, this is probably a bug", unit_type),
        }
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        use UnitType::*;
        output.write_u8(match self {
            Static(_) => 10,
            Tree(_) => 15,
            Animated(_) => 20,
            Doppleganger(_) => 25,
            Moving(_) => 30,
            Action(_) => 40,
            BaseCombat(_) => 50,
            Missile(_) => 60,
            Combat(_) => 70,
            Building(_) => 80,
        })?;

        match self {
            Static(unit) => unit.write_to(output)?,
            Tree(unit) => unit.write_to(output)?,
            Animated(unit) => unit.write_to(output)?,
            Doppleganger(unit) => unit.write_to(output)?,
            Moving(unit) => unit.write_to(output)?,
            Action(unit) => unit.write_to(output)?,
            BaseCombat(unit) => unit.write_to(output)?,
            Missile(unit) => unit.write_to(output)?,
            Combat(unit) => unit.write_to(output)?,
            Building(unit) => unit.write_to(output)?,
        }

        Ok(())
    }

    pub fn static_unit(&self) -> &StaticUnitType {
        use UnitType::*;
        match self {
            Static(unit) => unit,
            Tree(TreeUnitType(unit)) => unit,
            Animated(AnimatedUnitType {
                superclass: unit, ..
            }) => unit,
            Doppleganger(DopplegangerUnitType(AnimatedUnitType {
                superclass: unit, ..
            })) => unit,
            Moving(MovingUnitType {
                superclass: unit, ..
            }) => &unit.superclass,
            Action(ActionUnitType {
                superclass: unit, ..
            }) => &unit.superclass.superclass,
            BaseCombat(BaseCombatUnitType {
                superclass: unit, ..
            }) => &unit.superclass.superclass.superclass,
            Missile(MissileUnitType {
                superclass: unit, ..
            }) => &unit.superclass.superclass.superclass.superclass,
            Combat(CombatUnitType {
                superclass: unit, ..
            }) => &unit.superclass.superclass.superclass.superclass,
            Building(BuildingUnitType {
                superclass: unit, ..
            }) => &unit.superclass.superclass.superclass.superclass.superclass,
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
    attributes: ArrayVec<[UnitAttribute; 3]>,
    damage_sprites: Vec<DamageSprite>,
    selected_sound: Option<SoundID>,
    death_sound: Option<SoundID>,
    attack_reaction: u8,
    convert_terrain_flag: u8,
    copy_id: u16,
    unit_group: u16,
}

impl StaticUnitType {
    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
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
        unit_type.radius = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
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
        unit_type.object_flags = if version.as_f32() < 11.55 {
            0
        } else {
            input.read_u32::<LE>()?
        };
        unit_type.civilization = input.read_u8()?;
        unit_type.attribute_piece = input.read_u8()?;
        unit_type.outline_radius = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        for _ in 0..3 {
            let attr = UnitAttribute::from(input)?;
            if attr.attribute_type != 0xFFFF {
                unit_type.attributes.push(attr);
            }
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
        output.write_i16::<LE>(
            self.string_id2
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_u16::<LE>(self.unit_class)?;
        output.write_i16::<LE>(
            self.standing_sprite_1
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.standing_sprite_2
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.dying_sprite
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.undead_sprite
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_u8(self.undead_flag)?;
        output.write_u16::<LE>(self.hp)?;
        output.write_f32::<LE>(self.los)?;
        output.write_u8(self.garrison_capacity)?;
        output.write_f32::<LE>(self.radius.0)?;
        output.write_f32::<LE>(self.radius.1)?;
        output.write_f32::<LE>(self.radius.2)?;
        output.write_i16::<LE>(
            self.train_sound
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.damage_sound
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.death_spawn
                .map(|id| id.0.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_u8(self.sort_number)?;
        output.write_u8(if self.can_be_built_on { 1 } else { 0 })?;
        output.write_i16::<LE>(
            self.button_picture
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_u8(if self.hide_in_scenario_editor { 1 } else { 0 })?;
        output.write_i16::<LE>(
            self.portrait_picture
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
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
        for index in 0..self.attributes.capacity() {
            let attr = self
                .attributes
                .get(index)
                .cloned()
                .unwrap_or(UnitAttribute::default());
            attr.write_to(output)?;
        }
        output.write_u8(self.damage_sprites.len().try_into().unwrap())?;
        for sprite in &self.damage_sprites {
            sprite.write_to(output)?;
        }
        output.write_i16::<LE>(
            self.selected_sound
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.death_sound
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
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
    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        StaticUnitType::from(input, version).map(Self)
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
    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        Ok(Self {
            superclass: StaticUnitType::from(input, version)?,
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
    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        AnimatedUnitType::from(input, version).map(Self)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        self.0.write_to(output)
    }
}

#[derive(Debug, Default, Clone)]
pub struct MovingUnitType {
    superclass: AnimatedUnitType,
    move_sprite: Option<SpriteID>,
    run_sprite: Option<SpriteID>,
    turn_speed: f32,
    size_class: u8,
    trailing_unit: Option<UnitTypeID>,
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
    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: AnimatedUnitType::from(input, version)?,
            ..Default::default()
        };
        unit_type.move_sprite = read_opt_u16(input)?.map_into();
        unit_type.run_sprite = read_opt_u16(input)?.map_into();
        unit_type.turn_speed = input.read_f32::<LE>()?;
        unit_type.size_class = input.read_u8()?;
        unit_type.trailing_unit = read_opt_u16(input)?.map_into();
        unit_type.trailing_options = input.read_u8()?;
        unit_type.trailing_spacing = input.read_f32::<LE>()?;
        unit_type.move_algorithm = input.read_u8()?;
        unit_type.turn_radius = input.read_f32::<LE>()?;
        unit_type.turn_radius_speed = input.read_f32::<LE>()?;
        unit_type.maximum_yaw_per_second_moving = input.read_f32::<LE>()?;
        unit_type.stationary_yaw_revolution_time = input.read_f32::<LE>()?;
        unit_type.maximum_yaw_per_second_stationary = input.read_f32::<LE>()?;
        Ok(unit_type)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        self.superclass.write_to(output)?;
        output.write_i16::<LE>(
            self.move_sprite
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.run_sprite
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_f32::<LE>(self.turn_speed)?;
        output.write_u8(self.size_class)?;
        output.write_i16::<LE>(
            self.trailing_unit
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_u8(self.trailing_options)?;
        output.write_f32::<LE>(self.trailing_spacing)?;
        output.write_u8(self.move_algorithm)?;
        output.write_f32::<LE>(self.turn_radius)?;
        output.write_f32::<LE>(self.turn_radius_speed)?;
        output.write_f32::<LE>(self.maximum_yaw_per_second_moving)?;
        output.write_f32::<LE>(self.stationary_yaw_revolution_time)?;
        output.write_f32::<LE>(self.maximum_yaw_per_second_stationary)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ActionUnitType {
    superclass: MovingUnitType,
    default_task: Option<u16>,
    search_radius: f32,
    work_rate: f32,
    drop_site: Option<UnitTypeID>,
    backup_drop_site: Option<UnitTypeID>,
    task_by_group: u8,
    command_sound: Option<SoundID>,
    move_sound: Option<SoundID>,
    /// Task list for older versions; newer game versions store the task list at the root of the
    /// dat file, and use `unit_type.copy_id` to refer to one of those task lists.
    tasks: Option<TaskList>,
    run_pattern: u8,
}

impl ActionUnitType {
    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: MovingUnitType::from(input, version)?,
            ..Default::default()
        };
        unit_type.default_task = read_opt_u16(input)?;
        unit_type.search_radius = input.read_f32::<LE>()?;
        unit_type.work_rate = input.read_f32::<LE>()?;
        unit_type.drop_site = read_opt_u16(input)?.map_into();
        unit_type.backup_drop_site = read_opt_u16(input)?.map_into();
        unit_type.task_by_group = input.read_u8()?;
        unit_type.command_sound = read_opt_u16(input)?.map_into();
        unit_type.move_sound = read_opt_u16(input)?.map_into();
        unit_type.run_pattern = input.read_u8()?;
        Ok(unit_type)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        self.superclass.write_to(output)?;
        output.write_i16::<LE>(
            self.default_task
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_f32::<LE>(self.search_radius)?;
        output.write_f32::<LE>(self.work_rate)?;
        output.write_i16::<LE>(
            self.drop_site
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.backup_drop_site
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_u8(self.task_by_group)?;
        output.write_i16::<LE>(
            self.command_sound
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(
            self.move_sound
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_u8(self.run_pattern)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WeaponInfo {
    weapon_type: i16,
    value: i16,
}

impl WeaponInfo {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            weapon_type: input.read_i16::<LE>()?,
            value: input.read_i16::<LE>()?,
        })
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i16::<LE>(self.weapon_type)?;
        output.write_i16::<LE>(self.value)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct BaseCombatUnitType {
    superclass: ActionUnitType,
    base_armor: u16,
    weapons: Vec<WeaponInfo>,
    armors: Vec<WeaponInfo>,
    defense_terrain_bonus: Option<u16>,
    weapon_range_max: f32,
    area_effect_range: f32,
    attack_speed: f32,
    missile_id: Option<UnitTypeID>,
    base_hit_chance: i16,
    break_off_combat: i8,
    frame_delay: i16,
    weapon_offset: (f32, f32, f32),
    blast_level_offense: i8,
    weapon_range_min: f32,
    missed_missile_spread: f32,
    fight_sprite: Option<SpriteID>,
    displayed_armor: i16,
    displayed_attack: i16,
    displayed_range: f32,
    displayed_reload_time: f32,
}

impl BaseCombatUnitType {
    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: ActionUnitType::from(input, version)?,
            ..Default::default()
        };
        unit_type.base_armor = if version.as_f32() < 11.52 {
            input.read_u8()?.into()
        } else {
            input.read_u16::<LE>()?
        };
        let num_weapons = input.read_u16::<LE>()?;
        for _ in 0..num_weapons {
            unit_type.weapons.push(WeaponInfo::from(input)?);
        }
        let num_armors = input.read_u16::<LE>()?;
        for _ in 0..num_armors {
            unit_type.armors.push(WeaponInfo::from(input)?);
        }
        unit_type.defense_terrain_bonus = read_opt_u16(input)?;
        unit_type.weapon_range_max = input.read_f32::<LE>()?;
        unit_type.area_effect_range = input.read_f32::<LE>()?;
        unit_type.attack_speed = input.read_f32::<LE>()?;
        unit_type.missile_id = read_opt_u16(input)?.map_into();
        unit_type.base_hit_chance = input.read_i16::<LE>()?;
        unit_type.break_off_combat = input.read_i8()?;
        unit_type.frame_delay = input.read_i16::<LE>()?;
        unit_type.weapon_offset = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        unit_type.blast_level_offense = input.read_i8()?;
        unit_type.weapon_range_min = input.read_f32::<LE>()?;
        unit_type.missed_missile_spread = input.read_f32::<LE>()?;
        unit_type.fight_sprite = read_opt_u16(input)?.map_into();
        unit_type.displayed_armor = input.read_i16::<LE>()?;
        unit_type.displayed_attack = input.read_i16::<LE>()?;
        unit_type.displayed_range = input.read_f32::<LE>()?;
        unit_type.displayed_reload_time = input.read_f32::<LE>()?;
        Ok(unit_type)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        unimplemented!();
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
    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: BaseCombatUnitType::from(input, version)?,
            ..Default::default()
        };
        unit_type.missile_type = input.read_u8()?;
        unit_type.targetting_type = input.read_u8()?;
        unit_type.missile_hit_info = input.read_u8()?;
        unit_type.missile_die_info = input.read_u8()?;
        unit_type.area_effect_specials = input.read_u8()?;
        unit_type.ballistics_ratio = input.read_f32::<LE>()?;
        Ok(unit_type)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        self.superclass.write_to(output)?;
        output.write_u8(self.missile_type)?;
        output.write_u8(self.targetting_type)?;
        output.write_u8(self.missile_hit_info)?;
        output.write_u8(self.missile_die_info)?;
        output.write_u8(self.area_effect_specials)?;
        output.write_f32::<LE>(self.ballistics_ratio)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AttributeCost {
    attribute_type: i16,
    amount: i16,
    flag: u8,
}

impl AttributeCost {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let cost = Self {
            attribute_type: input.read_i16::<LE>()?,
            amount: input.read_i16::<LE>()?,
            flag: input.read_u8()?,
        };
        let _padding = input.read_u8()?;
        Ok(cost)
    }
    pub fn write_to<W: Write>(self, output: &mut W) -> Result<()> {
        output.write_i16::<LE>(self.attribute_type)?;
        output.write_i16::<LE>(self.amount)?;
        output.write_u8(self.flag)?;
        output.write_u8(0)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct CombatUnitType {
    superclass: BaseCombatUnitType,
    costs: ArrayVec<[AttributeCost; 3]>,
    create_time: u16,
    create_at_building: Option<UnitTypeID>,
    create_button: i8,
    rear_attack_modifier: f32,
    flank_attack_modifier: f32,
    hero_flag: u8,
    garrison_sprite: Option<SpriteID>,
    volley_fire_amount: f32,
    max_attacks_in_volley: i8,
    volley_spread: (f32, f32),
    volley_start_spread_adjustment: f32,
    volley_missile: Option<UnitTypeID>,
    special_attack_sprite: Option<SpriteID>,
    special_attack_flag: i8,
    displayed_pierce_armor: i16,
}

impl CombatUnitType {
    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: BaseCombatUnitType::from(input, version)?,
            ..Default::default()
        };

        for _ in 0..3 {
            let attr = AttributeCost::from(input)?;
            if attr.attribute_type >= 0 {
                unit_type.costs.push(attr);
            }
        }
        unit_type.create_time = input.read_u16::<LE>()?;
        unit_type.create_at_building = read_opt_u16(input)?.map_into();
        unit_type.create_button = input.read_i8()?;
        unit_type.rear_attack_modifier = input.read_f32::<LE>()?;
        unit_type.flank_attack_modifier = input.read_f32::<LE>()?;
        let _tribe_unit_type = input.read_u8()?;
        unit_type.hero_flag = input.read_u8()?;
        unit_type.garrison_sprite = {
            let n = input.read_i32::<LE>()?;
            if n < 0 {
                None
            } else {
                Some(n.try_into().unwrap())
            }
        };
        unit_type.volley_fire_amount = input.read_f32::<LE>()?;
        unit_type.max_attacks_in_volley = input.read_i8()?;
        unit_type.volley_spread = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        unit_type.volley_start_spread_adjustment = input.read_f32::<LE>()?;
        unit_type.volley_missile = {
            let n = input.read_i32::<LE>()?;
            if n == -1 {
                None
            } else {
                Some(n.try_into().unwrap())
            }
        };
        unit_type.special_attack_sprite = {
            let n = input.read_i32::<LE>()?;
            if n == -1 {
                None
            } else {
                Some(n.try_into().unwrap())
            }
        };
        unit_type.special_attack_flag = input.read_i8()?;
        unit_type.displayed_pierce_armor = input.read_i16::<LE>()?;

        Ok(unit_type)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct LinkedBuilding {
    unit_id: UnitTypeID,
    x_offset: f32,
    y_offset: f32,
}

impl LinkedBuilding {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            unit_id: input.read_u16::<LE>()?.into(),
            x_offset: input.read_f32::<LE>()?,
            y_offset: input.read_f32::<LE>()?,
        })
    }
    pub fn write_to<W: Write>(self, output: &mut W) -> Result<()> {
        output.write_u16::<LE>(self.unit_id.into())?;
        output.write_f32::<LE>(self.x_offset)?;
        output.write_f32::<LE>(self.y_offset)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct BuildingUnitType {
    superclass: CombatUnitType,
    construction_sprite: Option<SpriteID>,
    snow_sprite: Option<SpriteID>,
    connect_flag: u8,
    facet: i16,
    destroy_on_build: bool,
    on_build_make_unit: Option<UnitTypeID>,
    on_build_make_tile: Option<TerrainID>,
    on_build_make_overlay: i16,
    on_build_make_tech: Option<TechID>,
    can_burn: bool,
    linked_buildings: ArrayVec<[LinkedBuilding; 4]>,
    construction_unit: Option<UnitTypeID>,
    transform_unit: Option<UnitTypeID>,
    transform_sound: Option<SoundID>,
    construction_sound: Option<SoundID>,
    garrison_type: i8,
    garrison_heal_rate: f32,
    garrison_repair_rate: f32,
    salvage_unit: Option<UnitTypeID>,
    salvage_attributes: ArrayVec<[i8; 6]>,
}

impl BuildingUnitType {
    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: CombatUnitType::from(input, version)?,
            ..Default::default()
        };
        unit_type.construction_sprite = read_opt_u16(input)?.map_into();
        unit_type.snow_sprite = if version.as_f32() < 11.53 {
            None
        } else {
            read_opt_u16(input)?.map_into()
        };
        unit_type.connect_flag = input.read_u8()?;
        unit_type.facet = input.read_i16::<LE>()?;
        unit_type.destroy_on_build = input.read_u8()? != 0;
        unit_type.on_build_make_unit = read_opt_u16(input)?.map_into();
        unit_type.on_build_make_tile = read_opt_u16(input)?.map_into();
        unit_type.on_build_make_overlay = input.read_i16::<LE>()?;
        unit_type.on_build_make_tech = read_opt_u16(input)?.map_into();
        unit_type.can_burn = input.read_u8()? != 0;
        for _ in 0..unit_type.linked_buildings.capacity() {
            let link = LinkedBuilding::from(input)?;
            if link.unit_id.0 != 0xFFFF {
                unit_type.linked_buildings.push(link);
            }
        }

        unit_type.construction_unit = read_opt_u16(input)?.map_into();
        unit_type.transform_unit = read_opt_u16(input)?.map_into();
        unit_type.transform_sound = read_opt_u16(input)?.map_into();
        unit_type.construction_sound = read_opt_u16(input)?.map_into();
        unit_type.garrison_type = input.read_i8()?;
        unit_type.garrison_heal_rate = input.read_f32::<LE>()?;
        unit_type.garrison_repair_rate = input.read_f32::<LE>()?;
        unit_type.salvage_unit = read_opt_u16(input)?.map_into();
        for _ in 0..unit_type.salvage_attributes.capacity() {
            let attr = input.read_i8()?;
            unit_type.salvage_attributes.push(attr);
        }
        Ok(unit_type)
    }
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        Ok(())
    }
}
