//! Contains pure types, no IO.
//!
//! Most of these are more descriptive wrappers around integers.
use std::io::{Result, Error, ErrorKind};

/// SCX Format version.
pub type SCXVersion = [u8; 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiplomaticStance {
    Ally = 0,
    Neutral = 1,
    Enemy = 3,
}

impl DiplomaticStance {
    pub fn try_from(n: i32) -> Result<Self> {
        match n {
            0 => Ok(DiplomaticStance::Ally),
            1 => Ok(DiplomaticStance::Neutral),
            3 => Ok(DiplomaticStance::Enemy),
            _ => Err(Error::new(ErrorKind::Other, format!("invalid diplomatic stance {} (must be 0/1/3)", n))),
        }
    }
}

impl From<DiplomaticStance> for i32 {
    fn from(stance: DiplomaticStance) -> i32 {
        match stance {
            DiplomaticStance::Ally => 0,
            DiplomaticStance::Neutral => 1,
            DiplomaticStance::Enemy => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSet {
    BaseGame,
    Expansions,
}

impl DataSet {
    pub fn try_from(n: i32) -> Result<Self> {
        match n {
            0 => Ok(DataSet::BaseGame),
            1 => Ok(DataSet::Expansions),
            _ => Err(Error::new(ErrorKind::Other, "unknown data set")),
        }
    }
}

impl From<DataSet> for i32 {
    fn from(id: DataSet) -> i32 {
        match id {
            DataSet::BaseGame => 0,
            DataSet::Expansions => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DLCPackage {
    AgeOfKings,
    AgeOfConquerors,
    TheForgotten,
    AfricanKingdoms,
    RiseOfTheRajas,
}

impl DLCPackage {
    pub fn try_from(n: i32) -> Result<Self> {
        match n {
            2 => Ok(DLCPackage::AgeOfKings),
            3 => Ok(DLCPackage::AgeOfConquerors),
            4 => Ok(DLCPackage::TheForgotten),
            5 => Ok(DLCPackage::AfricanKingdoms),
            6 => Ok(DLCPackage::RiseOfTheRajas),
            _ => Err(Error::new(ErrorKind::Other, "unknown dlc package")),
        }
    }
}

impl From<DLCPackage> for i32 {
    fn from(dlc_id: DLCPackage) -> i32 {
        match dlc_id {
            DLCPackage::AgeOfKings => 2,
            DLCPackage::AgeOfConquerors => 3,
            DLCPackage::TheForgotten => 4,
            DLCPackage::AfricanKingdoms => 5,
            DLCPackage::RiseOfTheRajas => 6,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StartingAge {
    /// Use the game default.
    Default = -1,
    /// Start in the Dark Age with Nomad resources.
    Nomad = -2,
    /// Start in the Dark Age.
    DarkAge = 0,
    /// Start in the Feudal Age.
    FeudalAge = 1,
    /// Start in the Castle Age.
    CastleAge = 2,
    /// Start in the Imperial Age.
    ImperialAge = 3,
    /// Start in the Imperial Age with all technologies researched.
    PostImperialAge = 4,
}

impl StartingAge {
    /// Convert a starting age number to the appropriate enum for a particular
    /// data version.
    pub fn try_from(n: i32, version: f32) -> Result<Self> {
        if version < 1.25 {
            match n {
                -1 => Ok(StartingAge::Default),
                0 => Ok(StartingAge::DarkAge),
                1 => Ok(StartingAge::FeudalAge),
                2 => Ok(StartingAge::CastleAge),
                3 => Ok(StartingAge::ImperialAge),
                4 => Ok(StartingAge::PostImperialAge),
                _ => Err(Error::new(ErrorKind::Other, format!("invalid starting age {} (must be -1-4)", n))),
            }
        } else {
            match n {
                -1 | 0 => Ok(StartingAge::Default),
                1 => Ok(StartingAge::Nomad),
                2 => Ok(StartingAge::DarkAge),
                3 => Ok(StartingAge::FeudalAge),
                4 => Ok(StartingAge::CastleAge),
                5 => Ok(StartingAge::ImperialAge),
                6 => Ok(StartingAge::PostImperialAge),
                _ => Err(Error::new(ErrorKind::Other, format!("invalid starting age {} (must be -1-6)", n))),
            }
        }
    }
}
