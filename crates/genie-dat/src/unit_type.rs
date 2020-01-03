use crate::sound::SoundID;
use crate::sprite::{GraphicID, SpriteID};
use crate::task::TaskList;
use crate::terrain::TerrainID;
use crate::GameVersion;
use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
pub use genie_support::UnitTypeID;
use genie_support::{read_opt_u16, MapInto, StringKey, TechID};
use std::convert::TryInto;
use std::io::{self, Read, Result, Write};

pub type UnitClass = u16;

#[derive(Debug, Clone)]
pub enum UnitType {
    /// The base unit type, for units that do not do anything.
    Base(Box<BaseUnitType>),
    /// The tree unit type.
    Tree(Box<TreeUnitType>),
    /// Unit type that supports animated sprites.
    Animated(Box<AnimatedUnitType>),
    /// Unit type for the "fake" units you see in the fog of war, after the actual unit has been
    /// destroyed.
    Doppleganger(Box<DopplegangerUnitType>),
    /// Unit type that supports movement.
    Moving(Box<MovingUnitType>),
    /// Unit type that supports being tasked by a player.
    Action(Box<ActionUnitType>),
    /// Unit type that supports combat.
    BaseCombat(Box<BaseCombatUnitType>),
    /// Unit type for projectiles/missiles/arrows.
    Missile(Box<MissileUnitType>),
    /// Unit type that supports combat (with additional Age of Empires specific data).
    Combat(Box<CombatUnitType>),
    /// Unit type for buildings.
    Building(Box<BuildingUnitType>),
}

macro_rules! cast_unit_type {
    ($struct:ident, $tag:ident) => {
        impl From<$struct> for UnitType {
            fn from(v: $struct) -> Self {
                UnitType::$tag(Box::new(v))
            }
        }
    };
}

macro_rules! inherit_unit_type {
    ($struct:ident, $super:ident) => {
        impl std::ops::Deref for $struct {
            type Target = $super;
            fn deref(&self) -> &Self::Target {
                &self.superclass
            }
        }
    };
}

cast_unit_type!(BaseUnitType, Base);
cast_unit_type!(TreeUnitType, Tree);
cast_unit_type!(AnimatedUnitType, Animated);
cast_unit_type!(DopplegangerUnitType, Doppleganger);
cast_unit_type!(MovingUnitType, Moving);
cast_unit_type!(ActionUnitType, Action);
cast_unit_type!(BaseCombatUnitType, BaseCombat);
cast_unit_type!(MissileUnitType, Missile);
cast_unit_type!(CombatUnitType, Combat);
cast_unit_type!(BuildingUnitType, Building);

// inherit_unit_type!(TreeUnitType, BaseUnitType);
inherit_unit_type!(AnimatedUnitType, BaseUnitType);
// inherit_unit_type!(DopplegangerUnitType, AnimatedUnitType);
inherit_unit_type!(MovingUnitType, AnimatedUnitType);
inherit_unit_type!(ActionUnitType, MovingUnitType);
inherit_unit_type!(BaseCombatUnitType, ActionUnitType);
inherit_unit_type!(MissileUnitType, BaseCombatUnitType);
inherit_unit_type!(CombatUnitType, BaseCombatUnitType);
inherit_unit_type!(BuildingUnitType, CombatUnitType);

impl UnitType {
    fn type_id(&self) -> u8 {
        use UnitType::*;
        match self {
            Base(_) => 10,
            Tree(_) => 15,
            Animated(_) => 20,
            Doppleganger(_) => 25,
            Moving(_) => 30,
            Action(_) => 40,
            BaseCombat(_) => 50,
            Missile(_) => 60,
            Combat(_) => 70,
            Building(_) => 80,
        }
    }

    /// Read a unit type from an input stream.
    pub fn read_from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let unit_type = input.read_u8()?;
        match unit_type {
            10 => BaseUnitType::read_from(input, version).map_into(),
            15 => TreeUnitType::read_from(input, version).map_into(),
            20 => AnimatedUnitType::read_from(input, version).map_into(),
            25 => DopplegangerUnitType::read_from(input, version).map_into(),
            30 => MovingUnitType::read_from(input, version).map_into(),
            40 => ActionUnitType::read_from(input, version).map_into(),
            50 => BaseCombatUnitType::read_from(input, version).map_into(),
            60 => MissileUnitType::read_from(input, version).map_into(),
            70 => CombatUnitType::read_from(input, version).map_into(),
            80 => BuildingUnitType::read_from(input, version).map_into(),
            _ => panic!("unexpected unit type {}, this is probably a bug", unit_type),
        }
    }

    /// Write this unit type to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W, version: GameVersion) -> Result<()> {
        use UnitType::*;
        output.write_u8(self.type_id())?;

        match self {
            Base(unit) => unit.write_to(output, version)?,
            Tree(unit) => unit.write_to(output, version)?,
            Animated(unit) => unit.write_to(output, version)?,
            Doppleganger(unit) => unit.write_to(output, version)?,
            Moving(unit) => unit.write_to(output, version)?,
            Action(unit) => unit.write_to(output, version)?,
            BaseCombat(unit) => unit.write_to(output, version)?,
            Missile(unit) => unit.write_to(output, version)?,
            Combat(unit) => unit.write_to(output, version)?,
            Building(unit) => unit.write_to(output, version)?,
        }

        Ok(())
    }

    /// Get the base unit type properties for this unit.
    pub fn base(&self) -> &BaseUnitType {
        use UnitType::*;
        match self {
            Base(unit) => &unit,
            Tree(unit) => &unit.0,
            Animated(unit) => &unit,
            Doppleganger(unit) => &unit.0,
            Moving(unit) => &unit,
            Action(unit) => &unit,
            BaseCombat(unit) => &unit,
            Missile(unit) => &unit,
            Combat(unit) => &unit,
            Building(unit) => &unit,
        }
    }

    /// Get the animated unit type properties for this unit.
    pub fn animated(&self) -> Option<&AnimatedUnitType> {
        use UnitType::*;
        match self {
            Base(_) | Tree(_) => None,
            Animated(unit) => Some(&unit),
            Doppleganger(unit) => Some(&unit.0),
            Moving(unit) => Some(&unit),
            Action(unit) => Some(&unit),
            BaseCombat(unit) => Some(&unit),
            Missile(unit) => Some(&unit),
            Combat(unit) => Some(&unit),
            Building(unit) => Some(&unit),
        }
    }

    /// Get the moving unit type properties for this unit.
    pub fn moving(&self) -> Option<&MovingUnitType> {
        use UnitType::*;
        match self {
            Base(_) | Tree(_) | Animated(_) | Doppleganger(_) => None,
            Moving(unit) => Some(&unit),
            Action(unit) => Some(&unit),
            BaseCombat(unit) => Some(&unit),
            Missile(unit) => Some(&unit),
            Combat(unit) => Some(&unit),
            Building(unit) => Some(&unit),
        }
    }

    /// Get the action unit type properties for this unit.
    pub fn action(&self) -> Option<&ActionUnitType> {
        use UnitType::*;
        match self {
            Base(_) | Tree(_) | Animated(_) | Doppleganger(_) | Moving(_) => None,
            Action(unit) => Some(&unit),
            BaseCombat(unit) => Some(&unit),
            Missile(unit) => Some(&unit),
            Combat(unit) => Some(&unit),
            Building(unit) => Some(&unit),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UnitAttribute {
    pub attribute_type: u16,
    pub amount: f32,
    pub flag: u8,
}

impl UnitAttribute {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            attribute_type: input.read_u16::<LE>()?,
            amount: input.read_f32::<LE>()?,
            flag: input.read_u8()?,
        })
    }

    pub fn write_to<W: Write>(self, output: &mut W) -> Result<()> {
        output.write_u16::<LE>(self.attribute_type)?;
        output.write_f32::<LE>(self.amount)?;
        output.write_u8(self.flag)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct DamageSprite {
    pub sprite: SpriteID,
    pub damage_percent: u16,
    pub flag: u8,
}

impl DamageSprite {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
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
pub struct BaseUnitType {
    name: String,
    pub id: UnitTypeID,
    pub string_id: StringKey,
    string_id2: Option<StringKey>,
    pub unit_class: UnitClass,
    pub standing_sprite_1: Option<SpriteID>,
    pub standing_sprite_2: Option<SpriteID>,
    pub dying_sprite: Option<SpriteID>,
    pub undead_sprite: Option<SpriteID>,
    pub undead_flag: u8,
    pub hp: u16,
    pub los: f32,
    pub garrison_capacity: u8,
    pub radius: (f32, f32, f32),
    pub train_sound: Option<SoundID>,
    pub damage_sound: Option<SoundID>,
    pub death_spawn: Option<UnitTypeID>,
    pub sort_number: u8,
    pub can_be_built_on: bool,
    pub button_picture: Option<GraphicID>,
    pub hide_in_scenario_editor: bool,
    pub portrait_picture: Option<GraphicID>,
    pub enabled: bool,
    pub disabled: bool,
    pub tile_req: (i16, i16),
    pub center_tile_req: (i16, i16),
    pub construction_radius: (f32, f32),
    pub elevation_flag: bool,
    pub fog_flag: bool,
    pub terrain_restriction_id: u16,
    pub movement_type: u8,
    pub attribute_max_amount: u16,
    pub attribute_rot: f32,
    pub area_effect_level: u8,
    pub combat_level: u8,
    pub select_level: u8,
    pub map_draw_level: u8,
    pub unit_level: u8,
    pub multiple_attribute_mod: f32,
    pub map_color: u8,
    pub help_string_id: StringKey,
    pub help_page_id: u32,
    pub hotkey_id: u32,
    pub recyclable: bool,
    pub track_as_resource: bool,
    pub create_doppleganger: bool,
    pub resource_group: u8,
    pub occlusion_mask: u8,
    pub obstruction_type: u8,
    pub selection_shape: u8,
    pub object_flags: u32,
    pub civilization: u8,
    pub attribute_piece: u8,
    pub outline_radius: (f32, f32, f32),
    pub attributes: ArrayVec<[UnitAttribute; 3]>,
    pub damage_sprites: Vec<DamageSprite>,
    pub selected_sound: Option<SoundID>,
    pub death_sound: Option<SoundID>,
    pub attack_reaction: u8,
    pub convert_terrain_flag: u8,
    pub copy_id: u16,
    pub unit_group: u16,
}

impl BaseUnitType {
    pub fn read_from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
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
            let attr = UnitAttribute::read_from(input)?;
            if attr.attribute_type != 0xFFFF {
                unit_type.attributes.push(attr);
            }
        }
        unit_type.damage_sprites = {
            let num_damage_sprites = input.read_u8()?;
            let mut damage_sprites = vec![];
            for _ in 0..num_damage_sprites {
                damage_sprites.push(DamageSprite::read_from(input)?);
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

    /// Write this unit type to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W, version: GameVersion) -> Result<()> {
        output.write_u16::<LE>(self.id.into())?;
        output.write_i16::<LE>((&self.string_id).try_into().unwrap())?;
        write_opt_string_key(output, &self.string_id2)?;
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
                .map(|id| id.try_into().unwrap())
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
        output.write_u32::<LE>((&self.help_string_id).try_into().unwrap())?;
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
            let attr = self.attributes.get(index).cloned().unwrap_or_default();
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
pub struct TreeUnitType(BaseUnitType);

impl TreeUnitType {
    pub fn read_from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        BaseUnitType::read_from(input, version).map(Self)
    }

    /// Write this unit type to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W, version: GameVersion) -> Result<()> {
        self.0.write_to(output, version)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AnimatedUnitType {
    superclass: BaseUnitType,
    pub speed: f32,
}

impl AnimatedUnitType {
    pub fn read_from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        Ok(Self {
            superclass: BaseUnitType::read_from(input, version)?,
            speed: input.read_f32::<LE>()?,
        })
    }

    /// Write this unit type to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W, version: GameVersion) -> Result<()> {
        self.superclass.write_to(output, version)?;
        output.write_f32::<LE>(self.speed)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct DopplegangerUnitType(AnimatedUnitType);

impl DopplegangerUnitType {
    pub fn read_from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        AnimatedUnitType::read_from(input, version).map(Self)
    }

    /// Write this unit type to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W, version: GameVersion) -> Result<()> {
        self.0.write_to(output, version)
    }
}

#[derive(Debug, Default, Clone)]
pub struct MovingUnitType {
    superclass: AnimatedUnitType,
    pub move_sprite: Option<SpriteID>,
    pub run_sprite: Option<SpriteID>,
    pub turn_speed: f32,
    pub size_class: u8,
    pub trailing_unit: Option<UnitTypeID>,
    pub trailing_options: u8,
    pub trailing_spacing: f32,
    pub move_algorithm: u8,
    pub turn_radius: f32,
    pub turn_radius_speed: f32,
    pub maximum_yaw_per_second_moving: f32,
    pub stationary_yaw_revolution_time: f32,
    pub maximum_yaw_per_second_stationary: f32,
}

impl MovingUnitType {
    pub fn read_from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: AnimatedUnitType::read_from(input, version)?,
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

    /// Write this unit type to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W, version: GameVersion) -> Result<()> {
        self.superclass.write_to(output, version)?;
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
    pub default_task: Option<u16>,
    pub search_radius: f32,
    pub work_rate: f32,
    pub drop_site: Option<UnitTypeID>,
    pub backup_drop_site: Option<UnitTypeID>,
    pub task_by_group: u8,
    pub command_sound: Option<SoundID>,
    pub move_sound: Option<SoundID>,
    /// Task list for older versions; newer game versions store the task list at the root of the
    /// dat file, and use `unit_type.copy_id` to refer to one of those task lists.
    pub tasks: Option<TaskList>,
    pub run_pattern: u8,
}

impl ActionUnitType {
    pub fn read_from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: MovingUnitType::read_from(input, version)?,
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

    /// Write this unit type to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W, version: GameVersion) -> Result<()> {
        self.superclass.write_to(output, version)?;
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
    pub weapon_type: i16,
    pub value: i16,
}

impl WeaponInfo {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            weapon_type: input.read_i16::<LE>()?,
            value: input.read_i16::<LE>()?,
        })
    }
    pub fn write_to<W: Write>(self, output: &mut W) -> Result<()> {
        output.write_i16::<LE>(self.weapon_type)?;
        output.write_i16::<LE>(self.value)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct BaseCombatUnitType {
    superclass: ActionUnitType,
    pub base_armor: u16,
    pub weapons: Vec<WeaponInfo>,
    pub armors: Vec<WeaponInfo>,
    pub defense_terrain_bonus: Option<u16>,
    pub weapon_range_max: f32,
    pub area_effect_range: f32,
    pub attack_speed: f32,
    pub missile_id: Option<UnitTypeID>,
    pub base_hit_chance: i16,
    pub break_off_combat: i8,
    pub frame_delay: i16,
    pub weapon_offset: (f32, f32, f32),
    pub blast_level_offense: i8,
    pub weapon_range_min: f32,
    pub missed_missile_spread: f32,
    pub fight_sprite: Option<SpriteID>,
    pub displayed_armor: i16,
    pub displayed_attack: i16,
    pub displayed_range: f32,
    pub displayed_reload_time: f32,
}

impl BaseCombatUnitType {
    pub fn read_from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: ActionUnitType::read_from(input, version)?,
            ..Default::default()
        };
        unit_type.base_armor = if version.as_f32() < 11.52 {
            input.read_u8()?.into()
        } else {
            input.read_u16::<LE>()?
        };
        let num_weapons = input.read_u16::<LE>()?;
        for _ in 0..num_weapons {
            unit_type.weapons.push(WeaponInfo::read_from(input)?);
        }
        let num_armors = input.read_u16::<LE>()?;
        for _ in 0..num_armors {
            unit_type.armors.push(WeaponInfo::read_from(input)?);
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

    /// Write this unit type to an output stream.
    pub fn write_to<W: Write>(&self, _output: &mut W, version: GameVersion) -> Result<()> {
        unimplemented!();
    }
}

#[derive(Debug, Default, Clone)]
pub struct MissileUnitType {
    superclass: BaseCombatUnitType,
    pub missile_type: u8,
    pub targetting_type: u8,
    pub missile_hit_info: u8,
    pub missile_die_info: u8,
    pub area_effect_specials: u8,
    pub ballistics_ratio: f32,
}

impl MissileUnitType {
    /// Read this unit type from an input stream.
    pub fn read_from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: BaseCombatUnitType::read_from(input, version)?,
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

    /// Write this unit type to an output stream.
    pub fn write_to<W: Write>(&self, output: &mut W, version: GameVersion) -> Result<()> {
        self.superclass.write_to(output, version)?;
        output.write_u8(self.missile_type)?;
        output.write_u8(self.targetting_type)?;
        output.write_u8(self.missile_hit_info)?;
        output.write_u8(self.missile_die_info)?;
        output.write_u8(self.area_effect_specials)?;
        output.write_f32::<LE>(self.ballistics_ratio)?;
        Ok(())
    }
}

/// Resource cost for a unit.
#[derive(Debug, Default, Clone, Copy)]
pub struct AttributeCost {
    /// The player attribute type to give/take.
    pub attribute_type: i16,
    /// The amount of that attribute that should be taken/given.
    pub amount: i16,
    /// Flag determining how and when this cost is counted.
    ///
    /// TODO make this an enum
    pub flag: u8,
}

impl AttributeCost {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
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
    /// The costs of creating a unit of this type.
    pub costs: ArrayVec<[AttributeCost; 3]>,
    pub create_time: u16,
    /// Unit type ID of the building or unit where this unit can be created.
    pub create_at_building: Option<UnitTypeID>,
    /// Button location index where the button to create this unit should be shown when a
    /// `create_at_building` unit is selected.
    pub create_button: i8,
    pub rear_attack_modifier: f32,
    pub flank_attack_modifier: f32,
    /// Is this unit a hero unit?
    ///
    /// TODO what is special about hero units? Does it just opt into the healing behaviour?
    pub hero_flag: u8,
    pub garrison_sprite: Option<SpriteID>,
    pub volley_fire_amount: f32,
    pub max_attacks_in_volley: i8,
    pub volley_spread: (f32, f32),
    pub volley_start_spread_adjustment: f32,
    pub volley_missile: Option<UnitTypeID>,
    pub special_attack_sprite: Option<SpriteID>,
    pub special_attack_flag: i8,
    pub displayed_pierce_armor: i16,
}

impl CombatUnitType {
    /// Read this unit type from an input stream.
    pub fn read_from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: BaseCombatUnitType::read_from(input, version)?,
            ..Default::default()
        };

        for _ in 0..3 {
            let attr = AttributeCost::read_from(input)?;
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

    /// Write this unit type to an output stream.
    pub fn write_to<W: Write>(&self, _output: &mut W, version: GameVersion) -> Result<()> {
        unimplemented!();
    }
}

/// A linked, or "Annex" building. These allow for buildings made up of multiple pieces
/// with different behaviour, like the Town Centre with some walkable tiles and some non-walkable
/// tiles.
#[derive(Debug, Default, Clone)]
pub struct LinkedBuilding {
    /// Unit type ID for this linked building.
    pub unit_id: UnitTypeID,
    /// X offset in tiles from the centre of the "owner" building.
    pub x_offset: f32,
    /// Y offset in tiles from the centre of the "owner" building.
    pub y_offset: f32,
}

impl LinkedBuilding {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
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

/// Unit type class for buildings.
#[derive(Debug, Default, Clone)]
pub struct BuildingUnitType {
    superclass: CombatUnitType,
    /// Sprite to use during construction.
    pub construction_sprite: Option<SpriteID>,
    /// Sprite to use when this building is finished and built on snow.
    pub snow_sprite: Option<SpriteID>,
    /// TODO document
    pub connect_flag: u8,
    /// TODO document
    pub facet: i16,
    /// Whether the building should be immediately destroyed on completion.
    pub destroy_on_build: bool,
    /// Unit to spawn at the build site on completion.
    pub on_build_make_unit: Option<UnitTypeID>,
    /// Change the underlying terrain to this terrain ID on completion.
    pub on_build_make_tile: Option<TerrainID>,
    /// TODO document
    pub on_build_make_overlay: i16,
    /// Research this tech on completion.
    pub on_build_make_tech: Option<TechID>,
    /// Whether this building…can burn?
    ///
    /// TODO document the details
    pub can_burn: bool,
    pub linked_buildings: ArrayVec<[LinkedBuilding; 4]>,
    pub construction_unit: Option<UnitTypeID>,
    pub transform_unit: Option<UnitTypeID>,
    pub transform_sound: Option<SoundID>,
    pub construction_sound: Option<SoundID>,
    pub garrison_type: i8,
    pub garrison_heal_rate: f32,
    pub garrison_repair_rate: f32,
    pub salvage_unit: Option<UnitTypeID>,
    pub salvage_attributes: ArrayVec<[i8; 6]>,
}

impl BuildingUnitType {
    /// Read this unit type from an input stream.
    pub fn read_from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut unit_type = Self {
            superclass: CombatUnitType::read_from(input, version)?,
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
            let link = LinkedBuilding::read_from(input)?;
            if link.unit_id != 0xFFFF.into() {
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

    /// Write the unit type to an output stream.
    pub fn write_to<W: Write>(&self, _output: &mut W, version: GameVersion) -> Result<()> {
        unimplemented!()
    }
}

fn write_opt_string_key<W: Write>(output: &mut W, opt_key: &Option<StringKey>) -> Result<()> {
    output.write_i16::<LE>(if let Some(key) = opt_key {
        key.try_into()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
    } else {
        -1
    })?;
    Ok(())
}
