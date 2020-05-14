use crate::Result;
use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
pub use genie_dat::AttributeCost;
pub use genie_support::{StringKey, UnitTypeID};
use std::cmp;
use std::convert::{TryFrom, TryInto};

use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitBaseClass {
    Static = 10,
    Animated = 20,
    Doppelganger = 25,
    Moving = 30,
    Action = 40,
    BaseCombat = 50,
    Missile = 60,
    Combat = 70,
    Building = 80,
    Tree = 90,
}

impl cmp::PartialOrd for UnitBaseClass {
    fn partial_cmp(&self, other: &UnitBaseClass) -> Option<cmp::Ordering> {
        if self == other {
            return Some(cmp::Ordering::Equal);
        }

        let self_n = *self as u8;
        let other_n = *other as u8;

        // handle weird leaves specially
        match self {
            Self::Doppelganger => {
                if self_n > other_n {
                    Some(cmp::Ordering::Greater)
                } else {
                    None
                }
            }
            Self::Missile => {
                if self_n > other_n {
                    Some(cmp::Ordering::Greater)
                } else {
                    None
                }
            }
            Self::Tree => match other {
                Self::Static => Some(cmp::Ordering::Greater),
                _ => None,
            },
            _ => match other {
                Self::Doppelganger => {
                    if self_n < other_n {
                        Some(cmp::Ordering::Less)
                    } else {
                        None
                    }
                }
                Self::Missile => {
                    if self_n < other_n {
                        Some(cmp::Ordering::Less)
                    } else {
                        None
                    }
                }
                Self::Tree => match self {
                    Self::Static => Some(cmp::Ordering::Less),
                    _ => None,
                },
                _ => Some(self_n.cmp(&other_n)),
            },
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct CompactUnitType {
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

impl CompactUnitType {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let unit_base_class = input.read_u8()?.try_into().unwrap();
        let static_ = StaticUnitAttributes::read_from(&mut input)?;
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
            unit.animated = Some(AnimatedUnitAttributes::read_from(&mut input)?);
        }
        if unit_base_class >= UnitBaseClass::Moving {
            unit.moving = Some(MovingUnitAttributes::read_from(&mut input)?);
        }
        if unit_base_class >= UnitBaseClass::Action {
            unit.action = Some(ActionUnitAttributes::read_from(&mut input)?);
        }
        if unit_base_class >= UnitBaseClass::BaseCombat {
            unit.base_combat = Some(BaseCombatUnitAttributes::read_from(&mut input, version)?);
        }
        if unit_base_class >= UnitBaseClass::Missile {
            unit.missile = Some(MissileUnitAttributes::read_from(&mut input)?);
        }
        if unit_base_class >= UnitBaseClass::Combat {
            unit.combat = Some(CombatUnitAttributes::read_from(&mut input)?);
        }
        if unit_base_class >= UnitBaseClass::Building {
            unit.building = Some(BuildingUnitAttributes::read_from(&mut input)?);
        }
        Ok(unit)
    }
}

#[derive(Debug, Default, Clone)]
pub struct StaticUnitAttributes {
    id: UnitTypeID,
    copy_id: UnitTypeID,
    base_id: UnitTypeID,
    unit_class: u16,
    hotkey_id: u32,
    available: bool,
    death_object_id: Option<UnitTypeID>,
    string_id: Option<StringKey>,
    description_id: Option<StringKey>,
    flags: Option<u32>,
    help_string_id: Option<StringKey>,
    terrain_restriction: Option<u16>,
    hidden_in_editor: bool,
    is_queueable_tech: bool,
    hit_points: u16,
    line_of_sight: f32,
    garrison_capacity: u8,
    radius: (f32, f32),
    attribute_max_amount: u16,
    attribute_amount_held: f32,
    disabled: bool,
}

impl StaticUnitAttributes {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.id = input.read_u16::<LE>()?.into();
        attrs.copy_id = input.read_u16::<LE>()?.into();
        attrs.base_id = input.read_u16::<LE>()?.into();
        attrs.unit_class = input.read_u16::<LE>()?;
        attrs.hotkey_id = input.read_u32::<LE>()?;
        attrs.available = input.read_u8()? != 0;
        let hidden_in_editor = input.read_i8()?;
        // UserPatch data
        let hidden_flags = if hidden_in_editor == -16 {
            attrs.death_object_id = match input.read_i16::<LE>()? {
                -1 => None,
                id => Some(id.try_into().unwrap()),
            };
            attrs.string_id = Some(input.read_u16::<LE>()?.into());
            attrs.description_id = Some(input.read_u16::<LE>()?.into());
            attrs.flags = Some(input.read_u32::<LE>()?);
            attrs.help_string_id = Some(input.read_u32::<LE>()?.into());
            attrs.terrain_restriction = Some(input.read_u16::<LE>()?);
            input.read_i8()?
        } else {
            hidden_in_editor
        };
        attrs.hidden_in_editor = hidden_flags != 0;
        // Community Patch
        attrs.is_queueable_tech = (hidden_flags & 2) == 2;
        attrs.hit_points = input.read_u16::<LE>()?;
        attrs.line_of_sight = input.read_f32::<LE>()?;
        attrs.garrison_capacity = input.read_u8()?;
        attrs.radius = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        attrs.attribute_max_amount = input.read_u16::<LE>()?;
        attrs.attribute_amount_held = input.read_f32::<LE>()?;
        attrs.disabled = input.read_u8()? != 0;
        Ok(attrs)
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
pub struct MovingUnitAttributes {
    pub turn_speed: f32,
}

impl MovingUnitAttributes {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let turn_speed = input.read_f32::<LE>()?;
        Ok(Self { turn_speed })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_f32::<LE>(self.turn_speed)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ActionUnitAttributes {
    pub search_radius: f32,
    pub work_rate: f32,
}

impl ActionUnitAttributes {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let search_radius = input.read_f32::<LE>()?;
        let work_rate = input.read_f32::<LE>()?;
        Ok(Self {
            search_radius,
            work_rate,
        })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_f32::<LE>(self.search_radius)?;
        output.write_f32::<LE>(self.work_rate)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct HitType {
    pub hit_type: u16,
    pub amount: i16,
}

impl HitType {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let hit_type = input.read_u16::<LE>()?;
        let amount = input.read_i16::<LE>()?;
        Ok(Self { hit_type, amount })
    }
}

#[derive(Debug, Default, Clone)]
pub struct BaseCombatUnitAttributes {
    pub base_armor: u16,
    pub attacks: Vec<HitType>,
    pub armors: Vec<HitType>,
    pub attack_speed: f32,
    pub weapon_range_max: f32,
    pub base_hit_chance: u16,
    pub projectile_object_id: Option<UnitTypeID>,
    pub defense_terrain_bonus: Option<u16>,
    pub weapon_range_max_2: f32,
    pub area_of_effect: f32,
    pub weapon_range_min: f32,
}

impl BaseCombatUnitAttributes {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.base_armor = if version >= 11.52 {
            input.read_u16::<LE>()?
        } else {
            input.read_u8()?.into()
        };
        let num_attacks = input.read_u16::<LE>()?;
        for _ in 0..num_attacks {
            attrs.attacks.push(HitType::read_from(&mut input)?);
        }
        let num_armors = input.read_u16::<LE>()?;
        for _ in 0..num_armors {
            attrs.armors.push(HitType::read_from(&mut input)?);
        }
        attrs.attack_speed = input.read_f32::<LE>()?;
        attrs.weapon_range_max = input.read_f32::<LE>()?;
        attrs.base_hit_chance = input.read_u16::<LE>()?;
        attrs.projectile_object_id = match input.read_i16::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        attrs.defense_terrain_bonus = match input.read_i16::<LE>()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        attrs.weapon_range_max_2 = input.read_f32::<LE>()?;
        attrs.area_of_effect = input.read_f32::<LE>()?;
        attrs.weapon_range_min = input.read_f32::<LE>()?;
        Ok(attrs)
    }

    pub fn write_to(&self, _output: impl Write) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct MissileUnitAttributes {
    pub targeting_type: u8,
}

impl MissileUnitAttributes {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let targeting_type = input.read_u8()?;
        Ok(Self { targeting_type })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_u8(self.targeting_type)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct CombatUnitAttributes {
    pub costs: ArrayVec<[AttributeCost; 3]>,
    pub create_time: u16,
    pub original_pierce_armor: Option<u16>,
    pub original_armor: Option<u16>,
    pub original_weapon: Option<u16>,
    pub original_weapon_range: Option<f32>,
    pub area_effect_level: Option<u8>,
    pub frame_delay: Option<u16>,
    pub create_at_building: Option<UnitTypeID>,
    pub create_button: Option<i8>,
    pub rear_attack_modifier: Option<f32>,
    pub hero_flag: Option<u8>,
    pub volley_fire_amount: f32,
    pub max_attacks_in_volley: u8,
}

impl CombatUnitAttributes {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut attrs = Self::default();
        for _ in 0..3 {
            let attr = AttributeCost::read_from(&mut input)?;
            if attr.attribute_type >= 0 {
                attrs.costs.push(attr);
            }
        }
        let create_time = input.read_u16::<LE>()?;
        // UserPatch data
        if create_time == u16::max_value() - 15 {
            attrs.original_pierce_armor = Some(input.read_u16::<LE>()?);
            attrs.original_armor = Some(input.read_u16::<LE>()?);
            attrs.original_weapon = Some(input.read_u16::<LE>()?);
            attrs.original_weapon_range = Some(input.read_f32::<LE>()?);
            attrs.area_effect_level = Some(input.read_u8()?);
            attrs.frame_delay = Some(input.read_u16::<LE>()?);
            attrs.create_at_building = match input.read_i16::<LE>()? {
                -1 => None,
                id => Some(id.try_into().unwrap()),
            };
            attrs.create_button = Some(input.read_i8()?);
            attrs.rear_attack_modifier = Some(input.read_f32::<LE>()?);
            attrs.hero_flag = Some(input.read_u8()?);
            attrs.create_time = input.read_u16::<LE>()?;
        } else {
            attrs.create_time = create_time;
        };
        attrs.volley_fire_amount = input.read_f32::<LE>()?;
        attrs.max_attacks_in_volley = input.read_u8()?;
        Ok(attrs)
    }

    pub fn write_to(&self, _output: impl Write) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct BuildingUnitAttributes {
    pub facet: i16,
    // &facet + 62
    needs_a_name: Option<u8>,
    // &facet + 12
    needs_a_name_2: Option<u8>,
    pub garrison_heal_rate: Option<f32>,
}

impl BuildingUnitAttributes {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut attrs = Self::default();
        let facet = input.read_i16::<LE>()?;
        // UserPatch data
        if facet == 32767 {
            attrs.needs_a_name = Some(input.read_u8()?);
            attrs.needs_a_name_2 = Some(input.read_u8()?);
            attrs.garrison_heal_rate = Some(input.read_f32::<LE>()?);
            attrs.facet = input.read_i16::<LE>()?;
        } else {
            attrs.facet = facet;
        }
        Ok(attrs)
    }

    pub fn write_to(&self, _output: impl Write) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmp_unit_base_class() {
        assert!(UnitBaseClass::Static == UnitBaseClass::Static);
        assert!(UnitBaseClass::Static < UnitBaseClass::Animated);
        assert!(UnitBaseClass::Static < UnitBaseClass::Doppelganger);
        assert!(UnitBaseClass::Animated < UnitBaseClass::Doppelganger);
        assert!(!(UnitBaseClass::Moving < UnitBaseClass::Doppelganger));
        assert!(!(UnitBaseClass::Moving > UnitBaseClass::Doppelganger));
    }
}
