//! Contains pure types, no IO.
//!
//! Most of these are more descriptive wrappers around integers.
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};

/// The SCX Format version string. In practice, this does not really reflect the game version.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SCXVersion(pub(crate) [u8; 4]);

impl SCXVersion {
    /// Get the raw bytes representing this scx format version.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub(crate) fn to_player_version(self) -> Option<f32> {
        match self.as_bytes() {
            b"1.07" => Some(1.07),
            b"1.09" | b"1.10" | b"1.11" => Some(1.11),
            b"1.12" | b"1.13" | b"1.14" | b"1.15" | b"1.16" => Some(1.12),
            b"1.18" | b"1.19" => Some(1.13),
            b"1.20" | b"1.21" | b"1.32" | b"1.36" | b"1.37" => Some(1.14),
            _ => None,
        }
    }
}

impl Default for SCXVersion {
    fn default() -> Self {
        Self(*b"1.21")
    }
}

impl Debug for SCXVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", std::str::from_utf8(&self.0).unwrap())
    }
}

impl Display for SCXVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::str::from_utf8(&self.0).unwrap())
    }
}

impl PartialEq<[u8; 4]> for SCXVersion {
    fn eq(&self, other: &[u8; 4]) -> bool {
        other[0] == self.0[0] && other[1] == b'.' && other[2] == self.0[2] && other[3] == self.0[3]
    }
}

impl PartialEq<SCXVersion> for [u8; 4] {
    fn eq(&self, other: &SCXVersion) -> bool {
        other == self
    }
}

impl Ord for SCXVersion {
    fn cmp(&self, other: &SCXVersion) -> Ordering {
        match self.0[0].cmp(&other.0[0]) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.0[2].cmp(&other.0[2]) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.0[3].cmp(&other.0[3])
    }
}

impl PartialOrd for SCXVersion {
    fn partial_cmp(&self, other: &SCXVersion) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Could not parse a diplomatic stance because given number is an unknown stance ID.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("invalid diplomatic stance {} (must be 0/1/3)", .0)]
pub struct ParseDiplomaticStanceError(i32);

/// A player's diplomatic stance toward another player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiplomaticStance {
    /// The other player is an ally.
    Ally = 0,
    /// This player is neutral toward the other player.
    Neutral = 1,
    /// The other player is an enemy.
    Enemy = 3,
}

impl TryFrom<i32> for DiplomaticStance {
    type Error = ParseDiplomaticStanceError;

    fn try_from(n: i32) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(DiplomaticStance::Ally),
            1 => Ok(DiplomaticStance::Neutral),
            3 => Ok(DiplomaticStance::Enemy),
            n => Err(ParseDiplomaticStanceError(n)),
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

/// Could not parse a data set because given number is an unknown data set ID.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("invalid data set {} (must be 0/1)", .0)]
pub struct ParseDataSetError(i32);

/// The data set used by a scenario, HD Edition only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSet {
    /// The "base" data set, containing Age of Kings and the Age of Conquerors expansion.
    BaseGame,
    /// The "expansions" data set, containing the HD Edition expansions.
    Expansions,
}

impl TryFrom<i32> for DataSet {
    type Error = ParseDataSetError;
    fn try_from(n: i32) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(DataSet::BaseGame),
            1 => Ok(DataSet::Expansions),
            n => Err(ParseDataSetError(n)),
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

/// Could not parse a DLC package identifier because given number is an unknown DLC ID.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("unknown dlc package {}", .0)]
pub struct ParseDLCPackageError(i32);

/// An HD Edition DLC identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DLCPackage {
    /// The Age of Kings base game.
    AgeOfKings,
    /// The Age of Conquerors expansion.
    AgeOfConquerors,
    /// The Age of Kings base game. (HD DLC version, what's the difference?)
    DLCAgeOfKings,
    /// The Age of Conquerors expansion. (HD DLC version, what's the difference?)
    DLCAgeOfConquerors,
    /// The Forgotten expansion.
    TheForgotten,
    /// The African Kingdoms expansion.
    AfricanKingdoms,
    /// The Rise of the Rajas expansion.
    RiseOfTheRajas,
    /// The Last Khans expansion.
    LastKhans,
}

impl TryFrom<i32> for DLCPackage {
    type Error = ParseDLCPackageError;
    fn try_from(n: i32) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(DLCPackage::AgeOfKings),
            1 => Ok(DLCPackage::AgeOfConquerors),
            2 => Ok(DLCPackage::DLCAgeOfKings),
            3 => Ok(DLCPackage::DLCAgeOfConquerors),
            4 => Ok(DLCPackage::TheForgotten),
            5 => Ok(DLCPackage::AfricanKingdoms),
            6 => Ok(DLCPackage::RiseOfTheRajas),
            7 => Ok(DLCPackage::LastKhans),
            n => Err(ParseDLCPackageError(n)),
        }
    }
}

impl From<DLCPackage> for i32 {
    fn from(dlc_id: DLCPackage) -> i32 {
        match dlc_id {
            DLCPackage::AgeOfKings => 0,
            DLCPackage::AgeOfConquerors => 1,
            DLCPackage::DLCAgeOfKings => 2,
            DLCPackage::DLCAgeOfConquerors => 3,
            DLCPackage::TheForgotten => 4,
            DLCPackage::AfricanKingdoms => 5,
            DLCPackage::RiseOfTheRajas => 6,
            DLCPackage::LastKhans => 7,
        }
    }
}

fn expected_range(version: f32) -> &'static str {
    if version < 1.25 {
        "-1-4"
    } else {
        "-1-6"
    }
}

/// Could not parse a starting age because given number refers to an unknown age.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("invalid starting age {} (must be {})", .found, expected_range(*.version))]
pub struct ParseStartingAgeError {
    version: f32,
    found: i32,
}

/// The starting age.
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
    pub fn try_from(n: i32, version: f32) -> Result<Self, ParseStartingAgeError> {
        if version < 1.25 {
            match n {
                -1 => Ok(StartingAge::Default),
                0 => Ok(StartingAge::DarkAge),
                1 => Ok(StartingAge::FeudalAge),
                2 => Ok(StartingAge::CastleAge),
                3 => Ok(StartingAge::ImperialAge),
                4 => Ok(StartingAge::PostImperialAge),
                _ => Err(ParseStartingAgeError { version, found: n }),
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
                _ => Err(ParseStartingAgeError { version, found: n }),
            }
        }
    }

    /// Serialize the age identifier to an integer that is understood by the given game version.
    pub fn to_i32(self, version: f32) -> i32 {
        if version < 1.25 {
            match self {
                StartingAge::Default => -1,
                StartingAge::Nomad | StartingAge::DarkAge => 0,
                StartingAge::FeudalAge => 1,
                StartingAge::CastleAge => 2,
                StartingAge::ImperialAge => 3,
                StartingAge::PostImperialAge => 4,
            }
        } else {
            match self {
                StartingAge::Default => 0,
                StartingAge::Nomad => 1,
                StartingAge::DarkAge => 2,
                StartingAge::FeudalAge => 3,
                StartingAge::CastleAge => 4,
                StartingAge::ImperialAge => 5,
                StartingAge::PostImperialAge => 6,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VictoryCondition {
    Capture,
    Create,
    Destroy,
    DestroyMultiple,
    BringToArea,
    BringToObject,
    Attribute,
    Explore,
    CreateInArea,
    DestroyAll,
    DestroyPlayer,
    Points,
    Other(u8),
}

impl From<u8> for VictoryCondition {
    fn from(n: u8) -> Self {
        match n {
            0 => VictoryCondition::Capture,
            1 => VictoryCondition::Create,
            2 => VictoryCondition::Destroy,
            3 => VictoryCondition::DestroyMultiple,
            4 => VictoryCondition::BringToArea,
            5 => VictoryCondition::BringToObject,
            6 => VictoryCondition::Attribute,
            7 => VictoryCondition::Explore,
            8 => VictoryCondition::CreateInArea,
            9 => VictoryCondition::DestroyAll,
            10 => VictoryCondition::DestroyPlayer,
            11 => VictoryCondition::Points,
            n => VictoryCondition::Other(n),
        }
    }
}

impl From<VictoryCondition> for u8 {
    fn from(condition: VictoryCondition) -> Self {
        match condition {
            VictoryCondition::Capture => 0,
            VictoryCondition::Create => 1,
            VictoryCondition::Destroy => 2,
            VictoryCondition::DestroyMultiple => 3,
            VictoryCondition::BringToArea => 4,
            VictoryCondition::BringToObject => 5,
            VictoryCondition::Attribute => 6,
            VictoryCondition::Explore => 7,
            VictoryCondition::CreateInArea => 8,
            VictoryCondition::DestroyAll => 9,
            VictoryCondition::DestroyPlayer => 10,
            VictoryCondition::Points => 11,
            VictoryCondition::Other(n) => n,
        }
    }
}

/// All the versions an SCX file uses in a single struct.
#[derive(Debug, Clone, PartialEq)]
pub struct VersionBundle {
    /// The version of the 'container' file format.
    pub format: SCXVersion,
    /// The version of the header.
    pub header: u32,
    /// The version of the HD Edition DLC Options, only if `header` >= 3.
    pub dlc_options: Option<i32>,
    /// The compressed data version.
    pub data: f32,
    /// The version of embedded bitmaps.
    pub picture: u32,
    /// The version of the victory conditions data.
    pub victory: f32,
    /// The version of the trigger system.
    pub triggers: Option<f64>,
    /// The version of the map data.
    pub map: u32,
}

impl VersionBundle {
    /// A version bundle with the parameters AoE1 uses by default.
    pub fn aoe() -> Self {
        unimplemented!()
    }

    /// A version bundle with the parameters AoE1: Rise of Rome uses by default.
    pub fn ror() -> Self {
        Self {
            format: SCXVersion(*b"1.11"),
            header: 2,
            dlc_options: None,
            data: 1.15,
            picture: 1,
            victory: 2.0,
            triggers: None,
            map: 0,
        }
    }

    /// A version bundle with the parameters AoK uses by default.
    pub fn aok() -> Self {
        Self {
            format: SCXVersion(*b"1.18"),
            header: 2,
            dlc_options: None,
            data: 1.2,
            picture: 1,
            victory: 2.0,
            triggers: Some(1.6),
            map: 0,
        }
    }

    /// A version bundle with the parameters AoC uses by default
    pub fn aoc() -> Self {
        Self {
            format: SCXVersion(*b"1.21"),
            header: 2,
            dlc_options: None,
            data: 1.22,
            picture: 1,
            victory: 2.0,
            triggers: Some(1.6),
            map: 0,
        }
    }

    /// A version bundle with the parameters UserPatch 1.4 uses by default.
    pub fn userpatch_14() -> Self {
        Self::aoc()
    }

    /// A version bundle with the parameters UserPatch 1.5 uses by default.
    pub fn userpatch_15() -> Self {
        Self::userpatch_14()
    }

    /// A version bundle with the parameters HD Edition uses by default.
    pub fn hd_edition() -> Self {
        Self {
            format: SCXVersion(*b"1.21"),
            header: 3,
            dlc_options: Some(1000),
            data: 1.26,
            picture: 3,
            victory: 2.0,
            triggers: Some(1.6),
            map: 0,
        }
    }

    /// A version bundle with parameters Age of Empires 2: Definitive Edition uses by default.
    ///
    /// This will be updated along with DE2 patches.
    pub fn aoe2_de() -> Self {
        Self {
            format: SCXVersion(*b"1.37"),
            header: 5,
            dlc_options: Some(1000),
            data: 1.37,
            picture: 3,
            victory: 2.0,
            triggers: Some(2.2),
            map: 2,
        }
    }

    /// Returns whether this version is (likely) for an AoK scenario.
    pub fn is_aok(&self) -> bool {
        match self.format.as_bytes() {
            b"1.18" | b"1.19" | b"1.20" => true,
            _ => false,
        }
    }

    /// Returns whether this version is (likely) for an AoC scenario.
    pub fn is_aoc(&self) -> bool {
        self.format == *b"1.21" && self.data <= 1.22
    }

    /// Returns whether this version is (likely) for an HD Edition scenario.
    pub fn is_hd_edition(&self) -> bool {
        self.format == *b"1.21" || self.format == *b"1.22" && self.data > 1.22
    }

    /// Returns whether this version is (likely) for an AoE2: Definitive Edition scenario.
    pub fn is_age2_de(&self) -> bool {
        self.data >= 1.28
    }
}
