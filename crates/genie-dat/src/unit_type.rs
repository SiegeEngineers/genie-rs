use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Result, Write};

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

#[derive(Debug, Clone)]
pub struct StaticUnitType {}
#[derive(Debug, Clone)]
pub struct TreeUnitType {}
#[derive(Debug, Clone)]
pub struct AnimatedUnitType {}
#[derive(Debug, Clone)]
pub struct DopplegangerUnitType {}
#[derive(Debug, Clone)]
pub struct MovingUnitType {}
#[derive(Debug, Clone)]
pub struct ActionUnitType {}
#[derive(Debug, Clone)]
pub struct BaseCombatUnitType {}
#[derive(Debug, Clone)]
pub struct MissileUnitType {}
#[derive(Debug, Clone)]
pub struct CombatUnitType {}
#[derive(Debug, Clone)]
pub struct BuildingUnitType {}

impl StaticUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        unimplemented!()
    }
}

impl TreeUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        unimplemented!()
    }
}

impl AnimatedUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        unimplemented!()
    }
}

impl DopplegangerUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        unimplemented!()
    }
}

impl MovingUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        unimplemented!()
    }
}

impl ActionUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        unimplemented!()
    }
}

impl BaseCombatUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        unimplemented!()
    }
}

impl MissileUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        unimplemented!()
    }
}

impl CombatUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        unimplemented!()
    }
}

impl BuildingUnitType {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        unimplemented!()
    }
}
