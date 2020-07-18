//! Types related to unit types.

use crate::sound::SoundID;
use crate::sprite::{GraphicID, SpriteID};
use crate::task::TaskList;
use crate::terrain::TerrainID;
use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
pub use genie_support::UnitTypeID;
use genie_support::{read_opt_u16, read_opt_u32, MapInto, StringKey, TechID};
use std::cmp::{Ordering, PartialOrd};
use std::convert::{TryFrom, TryInto};
use std::io::{self, Read, Result, Write};

/// The base class of a unit indicates which data is available for that unit type.
///
/// # Comparison
/// This type implements a comparison operator. A base class value is greater-than-or-equal-to
/// another value if the base class "inherits" from the other value. For example, the
/// `Doppelganger` base class inherits from the `Animated` base class. No other class inherits from
/// `Doppelganger`. Therefore, it compares like this:
///
/// ```rust
/// # use genie_dat::unit_type::UnitBaseClass;
/// assert!(UnitBaseClass::Doppelganger > UnitBaseClass::Animated);
/// assert!(UnitBaseClass::Doppelganger == UnitBaseClass::Doppelganger);
/// assert_eq!(UnitBaseClass::Doppelganger < UnitBaseClass::Moving, false);
/// assert_eq!(UnitBaseClass::Doppelganger > UnitBaseClass::Moving, false);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UnitBaseClass {
    /// The base unit type, for units that do not do anything.
    Static = 10,
    /// Unit type that supports animated sprites.
    Animated = 20,
    /// Unit type for the "fake" units you see in the fog of war, after the actual unit has been
    /// destroyed.
    Doppelganger = 25,
    /// Unit type that supports movement.
    Moving = 30,
    /// Unit type that supports being tasked by a player.
    Action = 40,
    /// Unit type that supports combat.
    BaseCombat = 50,
    /// Unit type for projectiles/missiles/arrows.
    Missile = 60,
    /// Unit type that supports combat (with additional Age of Empires specific data).
    Combat = 70,
    /// Unit type for buildings.
    Building = 80,
    /// The tree unit type.
    Tree = 90,
}

impl PartialOrd for UnitBaseClass {
    fn partial_cmp(&self, other: &UnitBaseClass) -> Option<Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }

        let self_n = *self as u8;
        let other_n = *other as u8;

        // handle weird leaves specially
        match self {
            Self::Doppelganger => {
                if self_n > other_n {
                    Some(Ordering::Greater)
                } else {
                    None
                }
            }
            Self::Missile => {
                if self_n > other_n {
                    Some(Ordering::Greater)
                } else {
                    None
                }
            }
            Self::Tree => match other {
                Self::Static => Some(Ordering::Greater),
                _ => None,
            },
            _ => match other {
                Self::Doppelganger => {
                    if self_n < other_n {
                        Some(Ordering::Less)
                    } else {
                        None
                    }
                }
                Self::Missile => {
                    if self_n < other_n {
                        Some(Ordering::Less)
                    } else {
                        None
                    }
                }
                Self::Tree => match self {
                    Self::Static => Some(Ordering::Less),
                    _ => None,
                },
                _ => Some(self_n.cmp(&other_n)),
            },
        }
    }
}

/// An unexpected unit base class was found.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("unknown unit base class: {}", .0)]
pub struct ParseUnitBaseClassError(u8);

impl TryFrom<u8> for UnitBaseClass {
    type Error = ParseUnitBaseClassError;

    fn try_from(n: u8) -> std::result::Result<Self, Self::Error> {
        match n {
            10 => Ok(UnitBaseClass::Static),
            20 => Ok(UnitBaseClass::Animated),
            25 => Ok(UnitBaseClass::Doppelganger),
            30 => Ok(UnitBaseClass::Moving),
            40 => Ok(UnitBaseClass::Action),
            50 => Ok(UnitBaseClass::BaseCombat),
            60 => Ok(UnitBaseClass::Missile),
            70 => Ok(UnitBaseClass::Combat),
            80 => Ok(UnitBaseClass::Building),
            90 => Ok(UnitBaseClass::Tree),
            n => Err(ParseUnitBaseClassError(n)),
        }
    }
}

impl From<UnitBaseClass> for u8 {
    fn from(class: UnitBaseClass) -> u8 {
        class as u8
    }
}

/// A unit class, a group identifier for runtime behaviours.
pub type UnitClass = u16;

/// Data for a unit type.
///
/// Unit types have a [base class][] identifier that indicates which data is available for that
/// unit type. Data is split up into several `*Attributes` structs dictated by the unit's base class.
///
/// When editing the unit base class, the available attributes must also be updated. Failing to
/// keep the two in sync will cause a panic if you try to write the unit type data to a file or
/// other output.
///
/// [base class]: ./enum.UnitBaseClass.html
#[derive(Debug, Clone)]
pub struct UnitType {
    /// The base class for this unit type.
    pub unit_base_class: UnitBaseClass,
    /// The static unit type attributes: these are always available.
    pub static_: StaticUnitTypeAttributes,
    /// Animated unit type attributes, available if `self.unit_base_class >= UnitBaseClass::Animated`.
    pub animated: Option<AnimatedUnitTypeAttributes>,
    /// Moving unit type attributes, available if `self.unit_base_class >= UnitBaseClass::Moving`.
    pub moving: Option<MovingUnitTypeAttributes>,
    /// Action unit type attributes, available if `self.unit_base_class >= UnitBaseClass::Action`.
    pub action: Option<ActionUnitTypeAttributes>,
    /// BaseCombat unit type attributes, available if `self.unit_base_class >=
    /// UnitBaseClass::BaseCombat`.
    pub base_combat: Option<BaseCombatUnitTypeAttributes>,
    /// Missile unit type attributes, available if `self.unit_base_class >= UnitBaseClass::Missile`.
    pub missile: Option<MissileUnitTypeAttributes>,
    /// Combat unit type attributes, available if `self.unit_base_class >= UnitBaseClass::Combat`.
    pub combat: Option<CombatUnitTypeAttributes>,
    /// Building unit type attributes, available if `self.unit_base_class >=
    /// UnitBaseClass::Building`.
    pub building: Option<BuildingUnitTypeAttributes>,
}

impl UnitType {
    /// Read a unit type from an input stream.
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let unit_base_class = input.read_u8()?.try_into().unwrap();
        let static_ = StaticUnitTypeAttributes::read_from(&mut input, version)?;
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
            unit.animated = Some(AnimatedUnitTypeAttributes::read_from(&mut input, version)?);
        }
        if unit_base_class >= UnitBaseClass::Moving {
            unit.moving = Some(MovingUnitTypeAttributes::read_from(&mut input, version)?);
        }
        if unit_base_class >= UnitBaseClass::Action {
            unit.action = Some(ActionUnitTypeAttributes::read_from(&mut input, version)?);
        }
        if unit_base_class >= UnitBaseClass::BaseCombat {
            unit.base_combat = Some(BaseCombatUnitTypeAttributes::read_from(
                &mut input, version,
            )?);
        }
        if unit_base_class >= UnitBaseClass::Missile {
            unit.missile = Some(MissileUnitTypeAttributes::read_from(&mut input, version)?);
        }
        if unit_base_class >= UnitBaseClass::Combat {
            unit.combat = Some(CombatUnitTypeAttributes::read_from(&mut input, version)?);
        }
        if unit_base_class >= UnitBaseClass::Building {
            unit.building = Some(BuildingUnitTypeAttributes::read_from(&mut input, version)?);
        }
        Ok(unit)
    }

    /// Write this unit type to an output stream.
    ///
    /// # Panics
    /// This function panics when trying to write a unit type whose `unit_base_class` property does
    /// not match the available data attributes. For example, when `self.unit_base_class` is
    /// `UnitBaseClass::Animated`, but `self.animated` is `None`.
    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        output.write_u8(self.unit_base_class.into())?;

        self.static_.write_to(&mut output, version)?;

        if self.unit_base_class >= UnitBaseClass::Animated {
            self.animated
                .as_ref()
                .expect("Unit's base class was Animated, but it has no Animated attributes")
                .write_to(&mut output, version)?;
        } else {
            assert!(self.animated.is_none(), "Unexpected Animated attributes in a unit type whose base class does not support it");
        }

        if self.unit_base_class >= UnitBaseClass::Moving {
            self.moving
                .as_ref()
                .expect("Unit's base class was Moving, but it has no Moving attributes")
                .write_to(&mut output, version)?;
        } else {
            assert!(
                self.moving.is_none(),
                "Unexpected Moving attributes in a unit type whose base class does not support it"
            );
        }

        if self.unit_base_class >= UnitBaseClass::Action {
            self.action
                .as_ref()
                .expect("Unit's base class was Action, but it has no Action attributes")
                .write_to(&mut output, version)?;
        } else {
            assert!(
                self.action.is_none(),
                "Unexpected Action attributes in a unit type whose base class does not support it"
            );
        }

        if self.unit_base_class >= UnitBaseClass::BaseCombat {
            self.base_combat
                .as_ref()
                .expect("Unit's base class was BaseCombat, but it has no BaseCombat attributes")
                .write_to(&mut output, version)?;
        } else {
            assert!(self.base_combat.is_none(), "Unexpected BaseCombat attributes in a unit type whose base class does not support it");
        }

        if self.unit_base_class >= UnitBaseClass::Missile {
            self.missile
                .as_ref()
                .expect("Unit's base class was Missile, but it has no Missile attributes")
                .write_to(&mut output, version)?;
        } else {
            assert!(
                self.missile.is_none(),
                "Unexpected Missile attributes in a unit type whose base class does not support it"
            );
        }

        if self.unit_base_class >= UnitBaseClass::Combat {
            self.combat
                .as_ref()
                .expect("Unit's base class was Combat, but it has no Combat attributes")
                .write_to(&mut output, version)?;
        } else {
            assert!(
                self.combat.is_none(),
                "Unexpected Combat attributes in a unit type whose base class does not support it"
            );
        }

        if self.unit_base_class >= UnitBaseClass::Building {
            self.building
                .as_ref()
                .expect("Unit's base class was Building, but it has no Building attributes")
                .write_to(&mut output, version)?;
        } else {
            assert!(self.building.is_none(), "Unexpected Building attributes in a unit type whose base class does not support it");
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UnitAttribute {
    pub attribute_type: u16,
    pub amount: f32,
    pub flag: u8,
}

impl UnitAttribute {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        Ok(Self {
            attribute_type: input.read_u16::<LE>()?,
            amount: input.read_f32::<LE>()?,
            flag: input.read_u8()?,
        })
    }

    pub fn write_to(self, mut output: impl Write) -> Result<()> {
        output.write_u16::<LE>(self.attribute_type)?;
        output.write_f32::<LE>(self.amount)?;
        output.write_u8(self.flag)?;
        Ok(())
    }

    fn write_empty(mut output: impl Write) -> Result<()> {
        output.write_u16::<LE>(0xFFFF)?;
        output.write_f32::<LE>(0.0)?;
        output.write_u8(0)?;
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
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        Ok(Self {
            sprite: input.read_u16::<LE>()?.into(),
            damage_percent: input.read_u16::<LE>()?,
            flag: input.read_u8()?,
        })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_u16::<LE>(self.sprite.into())?;
        output.write_u16::<LE>(self.damage_percent)?;
        output.write_u8(self.flag)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct StaticUnitTypeAttributes {
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

impl StaticUnitTypeAttributes {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut unit_type = Self::default();
        let name_len = input.read_u16::<LE>()?;
        unit_type.id = input.read_u16::<LE>()?.into();
        unit_type.string_id = input.read_u16::<LE>()?.into();
        unit_type.string_id2 = read_opt_u16(&mut input)?;
        unit_type.unit_class = input.read_u16::<LE>()?;
        unit_type.standing_sprite_1 = read_opt_u16(&mut input)?;
        unit_type.standing_sprite_2 = read_opt_u16(&mut input)?;
        unit_type.dying_sprite = read_opt_u16(&mut input)?;
        unit_type.undead_sprite = read_opt_u16(&mut input)?;
        unit_type.undead_flag = input.read_u8()?;
        unit_type.hp = input.read_u16::<LE>()?;
        unit_type.los = input.read_f32::<LE>()?;
        unit_type.garrison_capacity = input.read_u8()?;
        unit_type.radius = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        unit_type.train_sound = read_opt_u16(&mut input)?;
        unit_type.damage_sound = read_opt_u16(&mut input)?;
        unit_type.death_spawn = read_opt_u16(&mut input)?;
        unit_type.sort_number = input.read_u8()?;
        unit_type.can_be_built_on = input.read_u8()? != 0;
        unit_type.button_picture = read_opt_u16(&mut input)?;
        unit_type.hide_in_scenario_editor = input.read_u8()? != 0;
        unit_type.portrait_picture = read_opt_u16(&mut input)?;
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
        unit_type.object_flags = if version < 11.55 {
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
            let attr = UnitAttribute::read_from(&mut input)?;
            if attr.attribute_type != 0xFFFF {
                unit_type.attributes.push(attr);
            }
        }
        unit_type.damage_sprites = {
            let num_damage_sprites = input.read_u8()?;
            let mut damage_sprites = vec![];
            for _ in 0..num_damage_sprites {
                damage_sprites.push(DamageSprite::read_from(&mut input)?);
            }
            damage_sprites
        };
        unit_type.selected_sound = read_opt_u16(&mut input)?;
        unit_type.death_sound = read_opt_u16(&mut input)?;
        unit_type.attack_reaction = input.read_u8()?;
        unit_type.convert_terrain_flag = input.read_u8()?;
        unit_type.name = {
            // TODO use not-UTF8 for the name
            let mut bytes = vec![0; usize::from(name_len)];
            input.read_exact(&mut bytes)?;
            String::from_utf8(bytes.iter().cloned().take_while(|b| *b != 0).collect()).unwrap()
        };
        unit_type.copy_id = input.read_u16::<LE>()?;
        unit_type.unit_group = input.read_u16::<LE>()?;
        Ok(unit_type)
    }

    /// Write this unit type to an output stream.
    pub fn write_to(&self, mut output: impl Write, _version: f32) -> Result<()> {
        // TODO use not-UTF8 for the name
        output.write_u16::<LE>(self.name.len() as u16)?;
        output.write_u16::<LE>(self.id.into())?;
        output.write_i16::<LE>((&self.string_id).try_into().unwrap())?;
        write_opt_string_key(&mut output, &self.string_id2)?;
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
            match self.attributes.get(index) {
                Some(attr) => attr.write_to(&mut output)?,
                None => UnitAttribute::write_empty(&mut output)?,
            }
        }
        output.write_u8(self.damage_sprites.len().try_into().unwrap())?;
        for sprite in &self.damage_sprites {
            sprite.write_to(&mut output)?;
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
        output.write_all(self.name.as_bytes())?;
        output.write_u16::<LE>(self.copy_id)?;
        output.write_u16::<LE>(self.unit_group)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct AnimatedUnitTypeAttributes {
    pub speed: f32,
}

impl AnimatedUnitTypeAttributes {
    pub fn read_from(mut input: impl Read, _version: f32) -> Result<Self> {
        Ok(Self {
            speed: input.read_f32::<LE>()?,
        })
    }

    /// Write this unit type to an output stream.
    pub fn write_to(&self, mut output: impl Write, _version: f32) -> Result<()> {
        output.write_f32::<LE>(self.speed)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct MovingUnitTypeAttributes {
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

impl MovingUnitTypeAttributes {
    pub fn read_from(mut input: impl Read, _version: f32) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.move_sprite = read_opt_u16(&mut input)?;
        attrs.run_sprite = read_opt_u16(&mut input)?;
        attrs.turn_speed = input.read_f32::<LE>()?;
        attrs.size_class = input.read_u8()?;
        attrs.trailing_unit = read_opt_u16(&mut input)?;
        attrs.trailing_options = input.read_u8()?;
        attrs.trailing_spacing = input.read_f32::<LE>()?;
        attrs.move_algorithm = input.read_u8()?;
        attrs.turn_radius = input.read_f32::<LE>()?;
        attrs.turn_radius_speed = input.read_f32::<LE>()?;
        attrs.maximum_yaw_per_second_moving = input.read_f32::<LE>()?;
        attrs.stationary_yaw_revolution_time = input.read_f32::<LE>()?;
        attrs.maximum_yaw_per_second_stationary = input.read_f32::<LE>()?;
        Ok(attrs)
    }

    /// Write this unit type to an output stream.
    pub fn write_to(&self, mut output: impl Write, _version: f32) -> Result<()> {
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
pub struct ActionUnitTypeAttributes {
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

impl ActionUnitTypeAttributes {
    pub fn read_from(mut input: impl Read, _version: f32) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.default_task = read_opt_u16(&mut input)?;
        attrs.search_radius = input.read_f32::<LE>()?;
        attrs.work_rate = input.read_f32::<LE>()?;
        attrs.drop_site = read_opt_u16(&mut input)?;
        attrs.backup_drop_site = read_opt_u16(&mut input)?;
        attrs.task_by_group = input.read_u8()?;
        attrs.command_sound = read_opt_u16(&mut input)?;
        attrs.move_sound = read_opt_u16(&mut input)?;
        attrs.run_pattern = input.read_u8()?;
        Ok(attrs)
    }

    /// Write this unit type to an output stream.
    pub fn write_to(&self, mut output: impl Write, _version: f32) -> Result<()> {
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
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        Ok(Self {
            weapon_type: input.read_i16::<LE>()?,
            value: input.read_i16::<LE>()?,
        })
    }
    pub fn write_to(self, mut output: impl Write) -> Result<()> {
        output.write_i16::<LE>(self.weapon_type)?;
        output.write_i16::<LE>(self.value)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct BaseCombatUnitTypeAttributes {
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

impl BaseCombatUnitTypeAttributes {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.base_armor = if version < 11.52 {
            input.read_u8()?.into()
        } else {
            input.read_u16::<LE>()?
        };
        let num_weapons = input.read_u16::<LE>()?;
        for _ in 0..num_weapons {
            attrs.weapons.push(WeaponInfo::read_from(&mut input)?);
        }
        let num_armors = input.read_u16::<LE>()?;
        for _ in 0..num_armors {
            attrs.armors.push(WeaponInfo::read_from(&mut input)?);
        }
        attrs.defense_terrain_bonus = read_opt_u16(&mut input)?;
        attrs.weapon_range_max = input.read_f32::<LE>()?;
        attrs.area_effect_range = input.read_f32::<LE>()?;
        attrs.attack_speed = input.read_f32::<LE>()?;
        attrs.missile_id = read_opt_u16(&mut input)?;
        attrs.base_hit_chance = input.read_i16::<LE>()?;
        attrs.break_off_combat = input.read_i8()?;
        attrs.frame_delay = input.read_i16::<LE>()?;
        attrs.weapon_offset = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        attrs.blast_level_offense = input.read_i8()?;
        attrs.weapon_range_min = input.read_f32::<LE>()?;
        attrs.missed_missile_spread = input.read_f32::<LE>()?;
        attrs.fight_sprite = read_opt_u16(&mut input)?;
        attrs.displayed_armor = input.read_i16::<LE>()?;
        attrs.displayed_attack = input.read_i16::<LE>()?;
        attrs.displayed_range = input.read_f32::<LE>()?;
        attrs.displayed_reload_time = input.read_f32::<LE>()?;
        Ok(attrs)
    }

    /// Write this unit type to an output stream.
    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        if version < 11.52 {
            output.write_u8(self.base_armor.try_into().unwrap())?;
        } else {
            output.write_u16::<LE>(self.base_armor)?;
        };
        output.write_u16::<LE>(self.weapons.len() as u16)?;
        for weapon in &self.weapons {
            weapon.write_to(&mut output)?;
        }
        output.write_u16::<LE>(self.armors.len() as u16)?;
        for armor in &self.armors {
            armor.write_to(&mut output)?;
        }
        output.write_u16::<LE>(self.defense_terrain_bonus.unwrap_or(0xFFFF))?;
        output.write_f32::<LE>(self.weapon_range_max)?;
        output.write_f32::<LE>(self.area_effect_range)?;
        output.write_f32::<LE>(self.attack_speed)?;
        output.write_u16::<LE>(self.missile_id.map_into().unwrap_or(0xFFFF))?;
        output.write_i16::<LE>(self.base_hit_chance)?;
        output.write_i8(self.break_off_combat)?;
        output.write_i16::<LE>(self.frame_delay)?;
        output.write_f32::<LE>(self.weapon_offset.0)?;
        output.write_f32::<LE>(self.weapon_offset.1)?;
        output.write_f32::<LE>(self.weapon_offset.2)?;
        output.write_i8(self.blast_level_offense)?;
        output.write_f32::<LE>(self.weapon_range_min)?;
        output.write_f32::<LE>(self.missed_missile_spread)?;
        output.write_u16::<LE>(self.fight_sprite.map_into().unwrap_or(0xFFFF))?;
        output.write_i16::<LE>(self.displayed_armor)?;
        output.write_i16::<LE>(self.displayed_attack)?;
        output.write_f32::<LE>(self.displayed_range)?;
        output.write_f32::<LE>(self.displayed_reload_time)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct MissileUnitTypeAttributes {
    pub missile_type: u8,
    pub targetting_type: u8,
    pub missile_hit_info: u8,
    pub missile_die_info: u8,
    pub area_effect_specials: u8,
    pub ballistics_ratio: f32,
}

impl MissileUnitTypeAttributes {
    /// Read this unit type from an input stream.
    pub fn read_from(mut input: impl Read, _version: f32) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.missile_type = input.read_u8()?;
        attrs.targetting_type = input.read_u8()?;
        attrs.missile_hit_info = input.read_u8()?;
        attrs.missile_die_info = input.read_u8()?;
        attrs.area_effect_specials = input.read_u8()?;
        attrs.ballistics_ratio = input.read_f32::<LE>()?;
        Ok(attrs)
    }

    /// Write this unit type to an output stream.
    pub fn write_to(&self, mut output: impl Write, _version: f32) -> Result<()> {
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
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let cost = Self {
            attribute_type: input.read_i16::<LE>()?,
            amount: input.read_i16::<LE>()?,
            flag: input.read_u8()?,
        };
        let _padding = input.read_u8()?;
        Ok(cost)
    }
    pub fn write_to(self, mut output: impl Write) -> Result<()> {
        output.write_i16::<LE>(self.attribute_type)?;
        output.write_i16::<LE>(self.amount)?;
        output.write_u8(self.flag)?;
        output.write_u8(0)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct CombatUnitTypeAttributes {
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

impl CombatUnitTypeAttributes {
    /// Read this unit type from an input stream.
    pub fn read_from(mut input: impl Read, _version: f32) -> Result<Self> {
        let mut attrs = Self::default();
        for _ in 0..3 {
            let attr = AttributeCost::read_from(&mut input)?;
            if attr.attribute_type >= 0 {
                attrs.costs.push(attr);
            }
        }
        attrs.create_time = input.read_u16::<LE>()?;
        attrs.create_at_building = read_opt_u16(&mut input)?;
        attrs.create_button = input.read_i8()?;
        attrs.rear_attack_modifier = input.read_f32::<LE>()?;
        attrs.flank_attack_modifier = input.read_f32::<LE>()?;
        let _tribe_unit_type = input.read_u8()?;
        attrs.hero_flag = input.read_u8()?;
        attrs.garrison_sprite = {
            let n = input.read_i32::<LE>()?;
            if n < 0 {
                None
            } else {
                Some(n.try_into().unwrap())
            }
        };
        attrs.volley_fire_amount = input.read_f32::<LE>()?;
        attrs.max_attacks_in_volley = input.read_i8()?;
        attrs.volley_spread = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        attrs.volley_start_spread_adjustment = input.read_f32::<LE>()?;
        attrs.volley_missile = read_opt_u32(&mut input)?;
        attrs.special_attack_sprite = read_opt_u32(&mut input)?;
        attrs.special_attack_flag = input.read_i8()?;
        attrs.displayed_pierce_armor = input.read_i16::<LE>()?;
        Ok(attrs)
    }

    /// Write this unit type to an output stream.
    pub fn write_to(&self, mut output: impl Write, _version: f32) -> Result<()> {
        for i in 0..3 {
            match self.costs.get(i) {
                Some(cost) => cost.write_to(&mut output)?,
                None => AttributeCost {
                    attribute_type: -1,
                    amount: 0,
                    flag: 0,
                }
                .write_to(&mut output)?,
            }
        }
        output.write_u16::<LE>(self.create_time)?;
        output.write_u16::<LE>(self.create_at_building.map_into().unwrap_or(0xFFFF))?;
        output.write_i8(self.create_button)?;
        output.write_f32::<LE>(self.rear_attack_modifier)?;
        output.write_f32::<LE>(self.flank_attack_modifier)?;
        output.write_u8(0)?;
        output.write_u8(self.hero_flag)?;
        output.write_u32::<LE>(self.garrison_sprite.map_into().unwrap_or(0xFFFF_FFFF))?;
        output.write_f32::<LE>(self.volley_fire_amount)?;
        output.write_i8(self.max_attacks_in_volley)?;
        output.write_f32::<LE>(self.volley_spread.0)?;
        output.write_f32::<LE>(self.volley_spread.1)?;
        output.write_f32::<LE>(self.volley_start_spread_adjustment)?;
        output.write_u32::<LE>(self.volley_missile.map_into().unwrap_or(0xFFFF_FFFF))?;
        output.write_u32::<LE>(self.special_attack_sprite.map_into().unwrap_or(0xFFFF_FFFF))?;
        output.write_i8(self.special_attack_flag)?;
        output.write_i16::<LE>(self.displayed_pierce_armor)?;
        Ok(())
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
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        Ok(Self {
            unit_id: input.read_u16::<LE>()?.into(),
            x_offset: input.read_f32::<LE>()?,
            y_offset: input.read_f32::<LE>()?,
        })
    }
    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_u16::<LE>(self.unit_id.into())?;
        output.write_f32::<LE>(self.x_offset)?;
        output.write_f32::<LE>(self.y_offset)?;
        Ok(())
    }
    fn write_empty(mut output: impl Write) -> Result<()> {
        output.write_u16::<LE>(0xFFFF)?;
        output.write_f32::<LE>(0.0)?;
        output.write_f32::<LE>(0.0)?;
        Ok(())
    }
}

/// Unit type class for buildings.
#[derive(Debug, Default, Clone)]
pub struct BuildingUnitTypeAttributes {
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

impl BuildingUnitTypeAttributes {
    /// Read this unit type from an input stream.
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.construction_sprite = read_opt_u16(&mut input)?;
        attrs.snow_sprite = if version < 11.53 {
            None
        } else {
            read_opt_u16(&mut input)?
        };
        attrs.connect_flag = input.read_u8()?;
        attrs.facet = input.read_i16::<LE>()?;
        attrs.destroy_on_build = input.read_u8()? != 0;
        attrs.on_build_make_unit = read_opt_u16(&mut input)?;
        attrs.on_build_make_tile = read_opt_u16(&mut input)?;
        attrs.on_build_make_overlay = input.read_i16::<LE>()?;
        attrs.on_build_make_tech = read_opt_u16(&mut input)?;
        attrs.can_burn = input.read_u8()? != 0;
        for _ in 0..attrs.linked_buildings.capacity() {
            let link = LinkedBuilding::read_from(&mut input)?;
            if link.unit_id != 0xFFFF.into() {
                attrs.linked_buildings.push(link);
            }
        }

        attrs.construction_unit = read_opt_u16(&mut input)?;
        attrs.transform_unit = read_opt_u16(&mut input)?;
        attrs.transform_sound = read_opt_u16(&mut input)?;
        attrs.construction_sound = read_opt_u16(&mut input)?;
        attrs.garrison_type = input.read_i8()?;
        attrs.garrison_heal_rate = input.read_f32::<LE>()?;
        attrs.garrison_repair_rate = input.read_f32::<LE>()?;
        attrs.salvage_unit = read_opt_u16(&mut input)?;
        for _ in 0..attrs.salvage_attributes.capacity() {
            let attr = input.read_i8()?;
            attrs.salvage_attributes.push(attr);
        }
        Ok(attrs)
    }

    /// Write the unit type to an output stream.
    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        output.write_u16::<LE>(self.construction_sprite.map_into().unwrap_or(0xFFFF))?;
        if version >= 11.53 {
            output.write_u16::<LE>(self.snow_sprite.map_into().unwrap_or(0xFFFF))?;
        }
        output.write_u8(self.connect_flag)?;
        output.write_i16::<LE>(self.facet)?;
        output.write_u8(if self.destroy_on_build { 1 } else { 0 })?;
        output.write_u16::<LE>(self.on_build_make_unit.map_into().unwrap_or(0xFFFF))?;
        output.write_u16::<LE>(self.on_build_make_tile.map_into().unwrap_or(0xFFFF))?;
        output.write_i16::<LE>(self.on_build_make_overlay)?;
        output.write_u16::<LE>(self.on_build_make_tech.map_into().unwrap_or(0xFFFF))?;
        output.write_u8(if self.can_burn { 1 } else { 0 })?;
        for i in 0..self.linked_buildings.capacity() {
            match self.linked_buildings.get(i) {
                Some(link) => link.write_to(&mut output)?,
                None => LinkedBuilding::write_empty(&mut output)?,
            }
        }
        output.write_u16::<LE>(self.construction_unit.map_into().unwrap_or(0xFFFF))?;
        output.write_u16::<LE>(self.transform_unit.map_into().unwrap_or(0xFFFF))?;
        output.write_u16::<LE>(self.transform_sound.map_into().unwrap_or(0xFFFF))?;
        output.write_u16::<LE>(self.construction_sound.map_into().unwrap_or(0xFFFF))?;
        output.write_i8(self.garrison_type)?;
        output.write_f32::<LE>(self.garrison_heal_rate)?;
        output.write_f32::<LE>(self.garrison_repair_rate)?;
        output.write_u16::<LE>(self.salvage_unit.map_into().unwrap_or(0xFFFF))?;
        for attr in &self.salvage_attributes {
            output.write_i8(*attr)?;
        }
        Ok(())
    }
}

fn write_opt_string_key(mut output: impl Write, opt_key: &Option<StringKey>) -> Result<()> {
    output.write_i16::<LE>(if let Some(key) = opt_key {
        key.try_into()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
    } else {
        -1
    })?;
    Ok(())
}
