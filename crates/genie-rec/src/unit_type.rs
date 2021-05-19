use crate::element::{ReadableHeaderElement, WritableHeaderElement};
use crate::reader::RecordingHeaderReader;
use crate::GameVariant::DefinitiveEdition;
use crate::Result;
use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
pub use genie_dat::unit_type::{AttributeCost, ParseUnitBaseClassError, UnitBaseClass};
use genie_support::{read_opt_u16, ReadSkipExt};
pub use genie_support::{StringKey, UnitTypeID};
use std::convert::TryInto;
use std::io::{Read, Write};

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

impl ReadableHeaderElement for CompactUnitType {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let unit_base_class = input.read_u8()?.try_into().unwrap();
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
    de: Option<StaticUnitAttributesDeExtension>,
}

#[derive(Debug, Default, Clone)]
pub struct StaticUnitAttributesDeExtension {
    name_id: StringKey,
    creation_id: StringKey,
    terrain_table: i16,
    dead_unit: Option<UnitTypeID>,
    icon: Option<i16>,
    blast_defense: u8,
}

impl ReadableHeaderElement for StaticUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.id = input.read_u16::<LE>()?.into();
        attrs.copy_id = input.read_u16::<LE>()?.into();
        attrs.base_id = input.read_u16::<LE>()?.into();
        if input.variant() >= DefinitiveEdition {
            // repeat of id??
            input.skip(2)?;
        }
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

        if input.variant() >= DefinitiveEdition {
            attrs.de = Some(StaticUnitAttributesDeExtension {
                name_id: input.read_u32::<LE>()?.into(),
                creation_id: input.read_u32::<LE>()?.into(),
                terrain_table: input.read_i16::<LE>()?,
                dead_unit: read_opt_u16(input)?,
                icon: read_opt_u16(input)?,
                blast_defense: input.read_u8()?,
            });
        }

        Ok(attrs)
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
pub struct MovingUnitAttributes {
    pub turn_speed: f32,
}

impl ReadableHeaderElement for MovingUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let turn_speed = input.read_f32::<LE>()?;
        Ok(Self { turn_speed })
    }
}

impl WritableHeaderElement for MovingUnitAttributes {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_f32::<LE>(self.turn_speed)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ActionUnitAttributes {
    pub search_radius: f32,
    pub work_rate: f32,
}

impl ReadableHeaderElement for ActionUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let search_radius = input.read_f32::<LE>()?;
        let work_rate = input.read_f32::<LE>()?;
        Ok(Self {
            search_radius,
            work_rate,
        })
    }
}

impl WritableHeaderElement for ActionUnitAttributes {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
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

impl ReadableHeaderElement for HitType {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
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
    pub de: Option<BaseCombatUnitAttributesDeExtension>,
}

#[derive(Debug, Default, Clone)]
pub struct BaseCombatUnitAttributesDeExtension {
    pub frame_delay: i16,
    pub blast_attack_level: u8,
    pub shown_melee_armor: i16,
    pub shown_attack: i16,
    pub shown_range: f32,
}

impl ReadableHeaderElement for BaseCombatUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut attrs = Self::default();
        attrs.base_armor = if input.version() >= 11.52 {
            input.read_u16::<LE>()?
        } else {
            input.read_u8()?.into()
        };
        let num_attacks = input.read_u16::<LE>()?;
        for _ in 0..num_attacks {
            attrs.attacks.push(HitType::read_from(input)?);
        }
        let num_armors = input.read_u16::<LE>()?;
        for _ in 0..num_armors {
            attrs.armors.push(HitType::read_from(input)?);
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

        if input.variant() >= DefinitiveEdition {
            attrs.de = Some(BaseCombatUnitAttributesDeExtension {
                frame_delay: input.read_i16::<LE>()?,
                blast_attack_level: input.read_u8()?,
                shown_melee_armor: input.read_i16::<LE>()?,
                shown_attack: input.read_i16::<LE>()?,
                shown_range: input.read_f32::<LE>()?,
            });
        }

        Ok(attrs)
    }
}

#[derive(Debug, Default, Clone)]
pub struct MissileUnitAttributes {
    pub targeting_type: u8,
}

impl ReadableHeaderElement for MissileUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let targeting_type = input.read_u8()?;
        Ok(Self { targeting_type })
    }
}

impl WritableHeaderElement for MissileUnitAttributes {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(self.targeting_type)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct CombatUnitAttributes {
    pub costs: ArrayVec<AttributeCost, 3>,
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
    pub de: Option<CombatUnitAttributesDeExtension>,
}

#[derive(Debug, Default, Clone)]
pub struct CombatUnitAttributesDeExtension {
    pub hero_flag: bool,
    pub shown_pierce_armor: i16,
    pub train_location: Option<i16>,
    pub train_button: u8,
    pub health_regen: f32,
}

impl ReadableHeaderElement for CombatUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut attrs = Self::default();
        for _ in 0..3 {
            let attr = AttributeCost::read_from(&mut *input)?;
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

        if input.variant() >= DefinitiveEdition {
            let mut de = CombatUnitAttributesDeExtension::default();
            let hero_flag = input.read_u8()?;
            attrs.hero_flag = Some(hero_flag);
            de.hero_flag = hero_flag == 1;
            de.train_location = read_opt_u16(input)?;
            de.train_button = input.read_u8()?;
            de.shown_pierce_armor = input.read_i16::<LE>()?;
            de.health_regen = input.read_f32::<LE>()?;
            attrs.de = Some(de);
        }

        Ok(attrs)
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

impl ReadableHeaderElement for BuildingUnitAttributes {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
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

        if input.variant() >= DefinitiveEdition {
            attrs.garrison_heal_rate = Some(input.read_f32::<LE>()?);
            // every item I've found so far is 0
            input.skip(4)?;
        }

        Ok(attrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmp_unit_base_class() {
        assert_eq!(UnitBaseClass::Static, UnitBaseClass::Static);
        assert!(UnitBaseClass::Static < UnitBaseClass::Animated);
        assert!(UnitBaseClass::Static < UnitBaseClass::Doppelganger);
        assert!(UnitBaseClass::Animated < UnitBaseClass::Doppelganger);
        assert!(!(UnitBaseClass::Moving < UnitBaseClass::Doppelganger));
        assert!(!(UnitBaseClass::Moving > UnitBaseClass::Doppelganger));
    }
}
