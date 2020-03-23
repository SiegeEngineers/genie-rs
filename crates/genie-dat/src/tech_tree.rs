use crate::civ::CivilizationID;
use crate::unit_type::UnitTypeID;
use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{read_opt_u32, TechID};
use std::convert::{TryFrom, TryInto};
use std::io::{self, Read, Result, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechTreeStatus {
    None,
    /// This building/unit/technology is available to the player.
    AvailablePlayer,
    /// This building/unit/technology is not available to the player.
    NotAvailablePlayer,
    /// Researching or constructing or creating.
    Researching,
    /// Researched or built.
    ResearchedCompleted,
    /// This building/unit/technology is available to the player if someone on their team is this
    /// civilization.
    AvailableTeam {
        civilization_id: CivilizationID,
    },
}

impl std::default::Default for TechTreeStatus {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParseTechTreeStatusError(u8);

impl std::fmt::Display for ParseTechTreeStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid tech tree node status {} (must be 1-5)", self.0)
    }
}

impl std::error::Error for ParseTechTreeStatusError {}

impl TryFrom<u8> for TechTreeStatus {
    type Error = ParseTechTreeStatusError;

    fn try_from(n: u8) -> std::result::Result<Self, Self::Error> {
        match n {
            1 => Ok(Self::None),
            2 => Ok(Self::AvailablePlayer),
            3 => Ok(Self::NotAvailablePlayer),
            4 => Ok(Self::Researching),
            5 => Ok(Self::ResearchedCompleted),
            10..=255 => Ok(Self::AvailableTeam {
                civilization_id: (n - 10).into(),
            }),
            n => Err(ParseTechTreeStatusError(n as u8)),
        }
    }
}

impl From<TechTreeStatus> for u8 {
    fn from(status: TechTreeStatus) -> Self {
        match status {
            TechTreeStatus::None => 1,
            TechTreeStatus::AvailablePlayer => 2,
            TechTreeStatus::NotAvailablePlayer => 3,
            TechTreeStatus::Researching => 4,
            TechTreeStatus::ResearchedCompleted => 5,
            TechTreeStatus::AvailableTeam { civilization_id } => u8::from(civilization_id) + 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechTreeType {
    None = 0,
    Age = 1,
    Unit = 2,
    UnitUpgrade = 3,
    Research = 4,
    BuildingTech = 5,
    BuildingNonTech = 6,
    UniqueUnit = 7,
}

impl std::default::Default for TechTreeType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParseTechTreeTypeError(i32);

impl std::fmt::Display for ParseTechTreeTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid tech tree node type {} (must be 0-7)", self.0)
    }
}

impl std::error::Error for ParseTechTreeTypeError {}

impl TryFrom<i32> for TechTreeType {
    type Error = ParseTechTreeTypeError;

    fn try_from(n: i32) -> std::result::Result<Self, Self::Error> {
        match n {
            0 => Ok(Self::None),
            1 => Ok(Self::Age),
            2 => Ok(Self::Unit),
            3 => Ok(Self::UnitUpgrade),
            4 => Ok(Self::Research),
            5 => Ok(Self::BuildingTech),
            6 => Ok(Self::BuildingNonTech),
            7 => Ok(Self::UniqueUnit),
            n => Err(ParseTechTreeTypeError(n)),
        }
    }
}

impl Into<i32> for TechTreeType {
    fn into(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Default, Clone)]
pub struct TechTree {
    pub ages: Vec<TechTreeAge>,
    pub buildings: Vec<TechTreeBuilding>,
    pub units: Vec<TechTreeUnit>,
    pub techs: Vec<TechTreeTech>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechTreeDependency {
    /// A dependency on an age being researched.
    ///
    /// This can typically only be 0-4, for Dark Age through Post-Imperial Age, or up to 5 in mods
    /// that add a new age. However the dat file format has space for 32 bits, and some data files
    /// in the wild contain incorrect data with much higher "age" IDs, so we have to follow suit.
    Age(i32),
    Building(UnitTypeID),
    Unit(UnitTypeID),
    Research(TechID),
}

impl TechTreeDependency {
    fn dependency_type(&self) -> TechTreeDependencyType {
        match self {
            Self::Age(_) => TechTreeDependencyType::Age,
            Self::Building(_) => TechTreeDependencyType::Building,
            Self::Unit(_) => TechTreeDependencyType::Unit,
            Self::Research(_) => TechTreeDependencyType::Research,
        }
    }

    fn raw_id(&self) -> i32 {
        match *self {
            Self::Age(id) => id.into(),
            Self::Building(id) => id.into(),
            Self::Unit(id) => id.into(),
            Self::Research(id) => {
                let id: u16 = id.into();
                id as i32
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TechTreeDependencyType {
    Age = 0,
    Building = 1,
    Unit = 2,
    Research = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct ParseTechTreeDependencyTypeError(i32);

impl std::fmt::Display for ParseTechTreeDependencyTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid tech tree dependency type {} (must be 0-3)",
            self.0
        )
    }
}

impl std::error::Error for ParseTechTreeDependencyTypeError {}

impl TryFrom<i32> for TechTreeDependencyType {
    type Error = ParseTechTreeDependencyTypeError;

    fn try_from(n: i32) -> std::result::Result<Self, Self::Error> {
        match n {
            0 => Ok(Self::Age),
            1 => Ok(Self::Building),
            2 => Ok(Self::Unit),
            3 => Ok(Self::Research),
            n => Err(ParseTechTreeDependencyTypeError(n)),
        }
    }
}

impl Into<i32> for TechTreeDependencyType {
    fn into(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Default, Clone)]
pub struct TechTreeDependencies(ArrayVec<[TechTreeDependency; 10]>);

#[derive(Debug, Default, Clone)]
pub struct TechTreeAge {
    age_id: i32,
    status: TechTreeStatus,
    node_type: TechTreeType,
    /// The buildings that become available in this age.
    pub dependent_buildings: Vec<UnitTypeID>,
    /// The units that become available in this age.
    pub dependent_units: Vec<UnitTypeID>,
    /// The techs that become available in this age.
    pub dependent_techs: Vec<TechID>,
    prerequisites: TechTreeDependencies,
    building_levels: u8,
    buildings_per_zone: [u8; 10],
    group_length_per_zone: [u8; 10],
    max_age_length: u8,
}

#[derive(Debug, Default, Clone)]
pub struct TechTreeBuilding {
    building_id: UnitTypeID,
    status: TechTreeStatus,
    node_type: TechTreeType,
    /// The tech ID that makes this building available. `None` if the building is available without
    /// requiring any techs.
    pub depends_tech_id: Option<TechID>,
    /// The buildings that become available by building this building.
    pub dependent_buildings: Vec<UnitTypeID>,
    /// The units that become available by building this building.
    pub dependent_units: Vec<UnitTypeID>,
    /// The techs that become available by building this building.
    pub dependent_techs: Vec<TechID>,
    prerequisites: TechTreeDependencies,
    /// ?
    level_no: u8,
    /// Total units and techs at this building by age, including ones that require research to
    /// unlock.
    total_children_by_age: [u8; 5],
    /// Initial units and techs at this building by age, excluding ones that require research to
    /// unlock.
    initial_children_by_age: [u8; 5],
}

#[derive(Debug, Default, Clone)]
pub struct TechTreeUnit {
    unit_id: UnitTypeID,
    status: TechTreeStatus,
    node_type: TechTreeType,
    depends_tech_id: Option<TechID>,
    building: UnitTypeID,
    requires_tech_id: Option<TechID>,
    dependent_units: Vec<UnitTypeID>,
    prerequisites: TechTreeDependencies,
    group_id: i32,
    level_no: i32,
}

#[derive(Debug, Default, Clone)]
pub struct TechTreeTech {
    tech_id: TechID,
    status: TechTreeStatus,
    node_type: TechTreeType,
    building: UnitTypeID,
    dependent_buildings: Vec<UnitTypeID>,
    dependent_units: Vec<UnitTypeID>,
    dependent_techs: Vec<TechID>,
    prerequisites: TechTreeDependencies,
    group_id: i32,
    level_no: i32,
}

impl TechTree {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let num_ages = input.read_u8()?;
        let num_buildings = input.read_u8()?;
        let num_units = input.read_u8()?;
        let num_techs = input.read_u8()?;
        let _num_groups = input.read_i32::<LE>()?;

        let mut ages = vec![];
        for _ in 0..num_ages {
            ages.push(TechTreeAge::read_from(input)?);
        }

        let mut buildings = vec![];
        for _ in 0..num_buildings {
            buildings.push(TechTreeBuilding::read_from(input)?);
        }

        let mut units = vec![];
        for _ in 0..num_units {
            units.push(TechTreeUnit::read_from(input)?);
        }

        let mut techs = vec![];
        for _ in 0..num_techs {
            techs.push(TechTreeTech::read_from(input)?);
        }

        Ok(Self {
            ages,
            buildings,
            units,
            techs,
        })
    }
}

impl TechTreeDependencies {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut deps = Self::default();
        let num = input.read_u8()?;
        let _padding = input.read_u8()?;
        let _padding = input.read_u8()?;
        let _padding = input.read_u8()?;

        let mut ids = [-1i32; 10];
        for id in ids.iter_mut() {
            *id = input.read_i32::<LE>()?;
        }
        let mut types = [-1i32; 10];
        for ty in types.iter_mut() {
            *ty = input.read_i32::<LE>()?;
        }

        for (&id, &ty) in ids.iter().zip(types.iter()).take(num as usize) {
            let dep_type: TechTreeDependencyType = ty.try_into().map_err(invalid_data)?;
            deps.0.push(match dep_type {
                TechTreeDependencyType::Age => {
                    TechTreeDependency::Age(id.try_into().map_err(invalid_data)?)
                }
                TechTreeDependencyType::Building => {
                    TechTreeDependency::Building(id.try_into().map_err(invalid_data)?)
                }
                TechTreeDependencyType::Unit => {
                    TechTreeDependency::Unit(id.try_into().map_err(invalid_data)?)
                }
                TechTreeDependencyType::Research => {
                    TechTreeDependency::Research(id.try_into().map_err(invalid_data)?)
                }
            });
        }

        Ok(deps)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        assert!(self.len() <= 10);
        output.write_u8(self.len() as u8)?;
        output.write_all(&[0, 0, 0])?;
        for i in 0..10 {
            output.write_i32::<LE>(self.0.get(i).map(TechTreeDependency::raw_id).unwrap_or(0))?;
        }
        for i in 0..10 {
            output.write_i32::<LE>(
                self.0
                    .get(i)
                    .map(TechTreeDependency::dependency_type)
                    .map(Into::into)
                    .unwrap_or(0),
            )?;
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &TechTreeDependency> {
        self.0.iter()
    }
}

/// Read a list of dependent "Thing" IDs for a tech tree node entry. The "Thing"
/// may be unit, building, tech IDs.
fn read_dependents<R, T>(input: &mut R) -> Result<Vec<T>>
where
    R: Read,
    T: TryFrom<i32>,
    // so `invalid_data` can convert it
    T::Error: std::error::Error + Send + Sync + 'static,
{
    let num = input.read_u8()?;
    let mut list = vec![];
    for _ in 0..num {
        list.push(input.read_i32::<LE>()?.try_into().map_err(invalid_data)?);
    }
    Ok(list)
}

impl TechTreeAge {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut age = Self::default();
        age.age_id = input.read_i32::<LE>()?;
        age.status = input.read_u8()?.try_into().map_err(invalid_data)?;
        age.dependent_buildings = read_dependents(input)?;
        age.dependent_units = read_dependents(input)?;
        age.dependent_techs = read_dependents(input)?;
        age.prerequisites = TechTreeDependencies::read_from(input)?;
        age.building_levels = input.read_u8()?;
        assert!(age.building_levels <= 10);
        for building in age.buildings_per_zone.iter_mut() {
            *building = input.read_u8()?;
        }
        for group_length in age.group_length_per_zone.iter_mut() {
            *group_length = input.read_u8()?;
        }
        age.max_age_length = input.read_u8()?;
        age.node_type = input.read_i32::<LE>()?.try_into().map_err(invalid_data)?;
        assert_eq!(age.node_type, TechTreeType::Age);
        Ok(age)
    }

    pub fn write_to<W: Write>(&self, _output: &mut W) -> Result<()> {
        unimplemented!();
        Ok(())
    }
}

impl TechTreeBuilding {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut building = Self::default();
        building.building_id = input.read_i32::<LE>()?.try_into().map_err(invalid_data)?;
        building.status = input.read_u8()?.try_into().map_err(invalid_data)?;
        building.dependent_buildings = read_dependents(input)?;
        building.dependent_units = read_dependents(input)?;
        building.dependent_techs = read_dependents(input)?;
        building.prerequisites = TechTreeDependencies::read_from(input)?;
        building.level_no = input.read_u8()?;
        for children in building.total_children_by_age.iter_mut() {
            *children = input.read_u8()?;
        }
        for children in building.initial_children_by_age.iter_mut() {
            *children = input.read_u8()?;
        }
        building.node_type = input.read_i32::<LE>()?.try_into().map_err(invalid_data)?;
        building.depends_tech_id = read_opt_u32(input)?;
        Ok(building)
    }

    pub fn write_to<W: Write>(&self, _output: &mut W) -> Result<()> {
        unimplemented!();
        Ok(())
    }
}

impl TechTreeUnit {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut unit = Self::default();
        unit.unit_id = input.read_i32::<LE>()?.try_into().map_err(invalid_data)?;
        unit.status = input.read_u8()?.try_into().map_err(invalid_data)?;
        unit.building = input.read_i32::<LE>()?.try_into().map_err(invalid_data)?;
        unit.prerequisites = TechTreeDependencies::read_from(input)?;
        unit.group_id = input.read_i32::<LE>()?;
        unit.dependent_units = read_dependents(input)?;
        unit.level_no = input.read_i32::<LE>()?;
        unit.requires_tech_id = read_opt_u32(input)?;
        unit.node_type = input.read_i32::<LE>()?.try_into().map_err(invalid_data)?;
        unit.depends_tech_id = read_opt_u32(input)?;
        Ok(unit)
    }

    pub fn write_to<W: Write>(&self, _output: &mut W) -> Result<()> {
        unimplemented!();
        Ok(())
    }
}

impl TechTreeTech {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut tech = Self::default();
        tech.tech_id = input.read_i32::<LE>()?.try_into().map_err(invalid_data)?;
        tech.status = input.read_u8()?.try_into().map_err(invalid_data)?;
        tech.building = input.read_i32::<LE>()?.try_into().map_err(invalid_data)?;
        tech.dependent_buildings = read_dependents(input)?;
        tech.dependent_units = read_dependents(input)?;
        tech.dependent_techs = read_dependents(input)?;
        tech.prerequisites = TechTreeDependencies::read_from(input)?;
        tech.group_id = input.read_i32::<LE>()?;
        tech.level_no = input.read_i32::<LE>()?;
        tech.node_type = input.read_i32::<LE>()?.try_into().map_err(invalid_data)?;
        Ok(tech)
    }

    pub fn write_to<W: Write>(&self, _output: &mut W) -> Result<()> {
        unimplemented!();
        Ok(())
    }
}

fn invalid_data<E: std::error::Error + Sized + Send + Sync + 'static>(err: E) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err)
}
