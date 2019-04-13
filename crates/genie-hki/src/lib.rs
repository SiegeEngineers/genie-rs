//! Reader/writer for Age of Empires 2 Hotkey info files.
//!
//! Hotkey files in AoE2 contain groups, each of which contain some number of
//! hotkeys. Hotkeys have a string ID, a keycode, and flags
//! for Ctrl/Alt/Shift modifiers. The index of the hotkey in its
//! group determines the action that will be taken when it is activated.

use std::error::Error;
use std::io::{self,Read, Write};
use std::fmt;
use std::slice::Iter;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};

/// Returns `Ok(id)`, where `id` is the id number of the string giving the text
/// representation of `keycode` in a language file.
/// Returns `None` if `keycode` is not represented in a language file.
pub fn keycode_id(keycode: i32) -> Option<i32> {
    match keycode {
        // TODO implement
        _ => None,
    }
}

/// Available hotkey groups.
pub enum HotkeyGroupId {
    UnitCommands = 0x0,
    GameCommands = 0x1,
    Scroll = 0x2,
    Villager = 0x3,
    TownCenter = 0x4,
    Dock = 0x5,
    Barracks = 0x6,
    ArcheryRange = 0x7,
    Stable = 0x8,
    SiegeWorkshop = 0x9,
    Monastery = 0xA,
    Market = 0xB,
    MilitaryUnits = 0xC,
    Castle = 0xD,
    Mill = 0xE,
}

/// Hotkeys for castles.
pub enum CastleHotkeys {
    Trebuchet = 0x0,
    UniqueUnit = 0x1,
    Petard = 0x2,
}

/// Hotkeys for military units.
pub enum MilitaryUnitHotkeys {
    Formation = 0x0,
    WheelLeft = 0x1,
    WheelRight = 0x2,
    AboutFace = 0x3,
    DisbandFormation = 0x4,
    Patrol = 0x5,
    Guard = 0x6,
    Follow = 0x7,
    Aggressive = 0x8,
    Defensive = 0x9,
    StandGround = 0xA,
    NoAttack = 0xB,
    /// HD Edition only.
    AttackMove = 0xC,
}

/// Hotkeys to change formations.
pub enum FormationHotkeys {
    Horde = 0x0,
    Box = 0x1,
    Line = 0x2,
    Staggered = 0x3,
    Flank = 0x4,
}

/// Hotkeys to control units.
pub enum UnitCommandHotkeys {
    BuildEconomic = 0x0,
    BuildMilitary = 0x1,
    Repair = 0x2,
    Group = 0x3,
    Ungroup = 0x4,
    Stop = 0x5,
    Unload = 0x6,
    Pack = 0x7,
    Unpack = 0x8,
    Heal = 0x9,
    Convert = 0xA,
    Garrison = 0xB,
    Delete = 0xC,
    SetGatherPoint = 0xD,
    AttackGround = 0xE,
}

/// Hotkeys for the game UI.
pub enum GameCommandHotkeys {
    CreateGroup0 = 0x0,
    CreateGroup1 = 0x1,
    CreateGroup2 = 0x2,
    CreateGroup3 = 0x3,
    CreateGroup4 = 0x4,
    CreateGroup5 = 0x5,
    CreateGroup6 = 0x6,
    CreateGroup7 = 0x7,
    CreateGroup8 = 0x8,
    CreateGroup9 = 0x9,
    SelectGroup0 = 0xA,
    SelectGroup1 = 0xB,
    SelectGroup2 = 0xC,
    SelectGroup3 = 0xD,
    SelectGroup4 = 0xE,
    SelectGroup5 = 0xF,
    SelectGroup6 = 0x10,
    SelectGroup7 = 0x11,
    SelectGroup8 = 0x12,
    SelectGroup9 = 0x13,
    Chat = 0x14,
    SpeedUp = 0x15,
    SpeedDown = 0x16,
    NextIdleVillager = 0x17,
    NextIdleVillager2 = 0x18,
    CycleFocusArea = 0x19,
    CycleFocusArea2 = 0x1A,
    GoToSelected = 0x1B,
    GoToTownCenter = 0x1C,
    GoToTownCenter2 = 0x1D,
    GoToMarket = 0x1E,
    ScrollChatDown = 0x1F,
    ScrollChatUp = 0x20,
    Score = 0x21,
    GoToBarracks = 0x22,
    GoToArcheryRange = 0x23,
    GoToStable = 0x24,
    GoToSiegeWorkshop = 0x25,
    GoToDock = 0x26,
    GoToMonastery = 0x27,
    GoToBlacksmith = 0x28,
    GoToMill = 0x29,
    GoToUniversity = 0x2A,
    TechTree = 0x2B,
    Achievements = 0x2C,
    DisplayGameTime = 0x2D,
    NextIdleMilitary = 0x2E,
    NextIdleMilitary2 = 0x2F,
    Flare = 0x30,
    GoToCastle = 0x31,
    GoToMiningCamp = 0x32,
    GoToSawMill = 0x33,
    MinimapCombat = 0x34,
    MinimapResource = 0x35,
    MinimapNormal = 0x36,
    ExtendedHelp = 0x37,
    AdvancedModes = 0x38,
    Diplomacy = 0x39,
    Menu = 0x3A,
    Objectives = 0x3B,
    ChatDialog = 0x3C,
    PauseGame = 0x3D,
    SaveGame = 0x3E,
    FriendFoeColors = 0x3F,
    PreviousView = 0x40,
    SaveChapter = 0x41,
}

/// Hotkeys for scrolling.
pub enum ScrollHotkeys {
    ScrollLeft = 0x0,
    ScrollRight = 0x1,
    ScrollUp = 0x2,
    ScrollDown = 0x3,
    FastScrollLeft = 0x4,
    FastScrollRight = 0x5,
    FastScrollUp = 0x6,
    FastScrollDown = 0x7,
    ScrollLeftUp = 0x8,
    ScrollLeftDown = 0x9,
    ScrollRightDown = 0xA,
    ScrollRightUp = 0xB,
    FastScrollLeftUp = 0xC,
    FastScrollLeftDown = 0xD,
    FastScrollRightDown = 0xE,
    FastScrollRightUp = 0xF,
}

/// Hotkeys for villagers.
pub enum VillagerHotkeys {
    BuildHouse = 0x0,
    BuildMill = 0x1,
    BuildBlacksmith = 0x2,
    BuildDock = 0x3,
    BuildBarracks = 0x4,
    BuildPalisadeWall = 0x5,
    BuildMarket = 0x6,
    BuildStoneWall = 0x7,
    BuildTower = 0x8,
    BuildBombardTower = 0x9,
    BuildGate1 = 0xA,
    BuildGate2 = 0xB,
    BuildFarm = 0xC,
    BuildArcheryRange = 0xD,
    BuildStable = 0xE,
    BuildMonastery = 0xF,
    BuildTownCenter = 0x10,
    BuildSiegeWorkshop = 0x11,
    BuildUniversity = 0x12,
    BuildWonder = 0x13,
    BuildCastle = 0x14,
    BuildTradeWorkshop = 0x15,
    BuildPackedTownCenter = 0x16,
    BuildSawMill = 0x17,
    BuildMiningCamp = 0x18,
    BuildFishTrap = 0x19,
    BuildOutpost = 0x1A,
    BuildNext = 0x1B,
    BuildPalisadeGate = 0x1C,
    BuildFeitoria = 0x1D,
}

/// Hotkeys for the town center.
pub enum TownCenterHotkeys {
    CreateVillager = 0x0,
    AgeAdvance = 0x1,
    CreateRaiderArcher = 0x2,
    CreateRaiderSwordsman = 0x3,
    CreateRaiderCavalry = 0x4,
    CreateRaiderCavArcher = 0x5,
    RingTownBell = 0x6,
    BackToWork = 0x7,
}

/// Hotkeys for mills.
pub enum MillHotkeys {
    AddFarm = 0x0,
}

/// Hotkeys for docks.
pub enum DockHotkeys {
    CreateFishingShip = 0x0,
    CreateTradeCog = 0x1,
    CreateGalley = 0x2,
    CreateCannonGalleon = 0x3,
    CreateFireGalley = 0x4,
    CreateDemolitionShip = 0x5,
    CreateBoardingGalley = 0x6,
    CreateTransport = 0x7,
    CreateLongboat = 0x8,
    CreateTurtleShip = 0x9,
}

/// Hotkeys for the barracks.
pub enum BarracksHotkeys {
    CreateMilitia = 0x0,
    CreatePikeman = 0x1,
    CreateEagleWarrior = 0x2,
    CreateHuskarl = 0x3,
}

/// Hotkeys for archery ranges.
pub enum ArcheryRangeHotkeys {
    CreateArcher = 0x0,
    CreateSkirmisher = 0x1,
    CreateCavArcher = 0x2,
    CreateHandCannoneer = 0x3,
    CreateGenitour = 0x4,
}

/// Hotkeys for stables.
pub enum StableHotkeys {
    CreateScout = 0x0,
    CreateCamel = 0x1,
    CreateKnight = 0x2,
    CreateBattleElephant = 0x3,
}

/// Hotkeys for siege workshops.
pub enum SiegeWorkshopHotkeys {
    CreateRam = 0x0,
    CreateScorpion = 0x1,
    CreateMangonel = 0x2,
    CreateBombardCannon = 0x3,
    CreateSiegeTower = 0x4,
}

/// Hotkeys for the monastery.
pub enum MonasteryHotkeys {
    CreateMonk = 0x0,
    CreateMissionary = 0x1,
}

/// Hotkeys for the market.
pub enum MarketHotkeys {
    CreateTradeCart = 0x0,
}

/// Hotkeys for the blacksmith.
pub enum BlacksmithHotkeys {
}

/// Represents an error when binding or unbinding a hotkey that doesn't exist.
#[derive(Debug)]
pub enum IndexError {
    /// Represents an index error when accessing a nonexistent group.
    GroupIndex(GroupIndexError),

    /// Represents an index error when accessing a nonexistent hotkey within a
    /// group.
    HotkeyIndex(HotkeyIndexError),
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IndexError::GroupIndex(err) => err.fmt(f),
            IndexError::HotkeyIndex(err) => err.fmt(f),
        }
    }
}

impl Error for IndexError { }

impl From<GroupIndexError> for IndexError {
    fn from(err: GroupIndexError) -> Self { IndexError::GroupIndex(err) }
}

impl From<HotkeyIndexError> for IndexError {
    fn from(err: HotkeyIndexError) -> Self { IndexError::HotkeyIndex(err) }
}

/// Represents an error when accessing a hotkey group that does not exist.
///
/// The first index represents the index of the group, and the second index
/// represents the number of groups. The first index must be greater than or
/// equal to the second index.
#[derive(Debug)]
pub struct GroupIndexError {
    /// The index of the group, must be greater than or equal to `num_groups`.
    index: usize,
    /// The number of groups, must be less than or equal to `index`.
    num_groups: usize,
}

impl GroupIndexError {
    /// Returns a `GroupIndexError` with group index `index` and a number of
    /// groups equal to `num_groups`.
    /// Panics if `index < num_groups`.
    pub fn new(index: usize, num_groups: usize) -> Self {
        assert!(num_groups <= index);
        Self { index, num_groups }
    }

    /// Returns the index of the group that was accessed.
    pub fn index(&self) -> usize { self.index }

    /// Returns the number of valid groups.
    pub fn num_groups(&self) -> usize { self.num_groups }
}

impl fmt::Display for GroupIndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Group id {} must be less than the number of groups {}.",
            self.index, self.num_groups)
    }
}

impl Error for GroupIndexError { }

/// Represents an error when accessing a hotkey that does not exist.
#[derive(Debug)]
pub struct HotkeyIndexError {
    /// The index of the hotkey, must be greater than or equal to `num_hotkeys`.
    index: usize,
    /// The number of hotkeys, must be less than or equal to `index`.
    num_hotkeys: usize,
}

impl HotkeyIndexError {
    /// Returns a `HotkeyIndexError` with hotkey index `index` and a number of
    /// hotkeys equal to `num_hotkeys`.
    pub fn new(index: usize, num_hotkeys: usize) -> Self {
        assert!(num_hotkeys <= index);
        Self { index, num_hotkeys }
    }

    /// Returns the index of the hotkey that was accessed.
    pub fn index(&self) -> usize { self.index }

    /// Returns the number of valid hotkeys.
    pub fn num_hotkeys(&self) -> usize { self.num_hotkeys }
}

impl fmt::Display for HotkeyIndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hotkey id {} must be less than the number of hotkeys {}.",
            self.index, self.num_hotkeys)
    }
}

impl Error for HotkeyIndexError { }

/// The information about a single hotkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hotkey {
    /// Keycode that activates this hotkey.
    ///
    /// You can use a crate like [keycodes](https://docs.rs/keycodes/0.1.0/)
    /// to compare this to named virtual keys like `keycodes::KEY_RETURN`.
    pub key: i32,
    /// The string ID for this hotkey's label. -1 if this hotkey is unused.
    pub string_id: i32,
    /// Whether the Ctrl key needs to be held to activate this hotkey.
    pub ctrl: bool,
    /// Whether the Alt key needs to be held to activate this hotkey.
    pub alt: bool,
    /// Whether the Shift key needs to be held to activate this hotkey.
    pub shift: bool,
    /// Not sure what this does yet? Actually may be unused…
    mouse: i8,
}

impl Default for Hotkey {
    fn default() -> Self {
        Self {
            key: 0,
            string_id: -1,
            ctrl: false,
            alt: false,
            shift: false,
            mouse: 0,
        }
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "{}: {}{}{}{}",
            self.string_id,
            if self.ctrl  { "Ctrl-"  } else { "" } ,
            if self.alt   { "Alt-"   } else { "" } ,
            if self.shift { "Shift-" } else { "" } ,
            self.key
        )
    }
}

impl Hotkey {
    pub fn key(self, key: i32) -> Self {
        Self { key, ..self }
    }

    pub fn string_id(self, string_id: i32) -> Self {
        Self { string_id, ..self }
    }

    pub fn ctrl(self, ctrl: bool) -> Self {
        Self { ctrl, ..self }
    }

    pub fn alt(self, alt: bool) -> Self {
        Self { alt, ..self }
    }

    pub fn shift(self, shift: bool) -> Self {
        Self { shift, ..self }
    }

    /// Read a hotkey from an input stream.
    pub(crate) fn from<R: Read>(input: &mut R) -> io::Result<Self> {
        let key = input.read_i32::<LE>()?;
        let string_id = input.read_i32::<LE>()?;
        let ctrl = input.read_u8()? != 0;
        let alt = input.read_u8()? != 0;
        let shift = input.read_u8()? != 0;
        let mouse = input.read_i8()?;

        Ok(Self { key, string_id, ctrl, alt, shift, mouse })
    }

    /// Write a hotkey to an output stream.
    pub(crate) fn write_to<W: Write>(&self, output: &mut W) -> io::Result<()> {
        output.write_i32::<LE>(self.key)?;
        output.write_i32::<LE>(self.string_id)?;
        output.write_u8(if self.ctrl { 1 } else { 0 })?;
        output.write_u8(if self.alt { 1 } else { 0 })?;
        output.write_u8(if self.shift { 1 } else { 0 })?;
        output.write_i8(self.mouse)?;
        Ok(())
    }
}

/// Represents a group of `Hotkey`s.
///
/// Different groups may have different numbers of hotkeys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyGroup {
    /// The hotkeys in this group, ordered by the order they appear in a
    /// hotkey file.
    hotkeys: Vec<Hotkey>,
}

impl HotkeyGroup {
    /// Read a hotkey group from an input stream.
    pub(crate) fn from<R: Read>(input: &mut R) -> io::Result<Self> {
        let num_hotkeys = input.read_u32::<LE>()?;
        let mut hotkeys = Vec::with_capacity(num_hotkeys as usize);
        for _ in 0..num_hotkeys {
            hotkeys.push(Hotkey::from(input)?);
        }

        Ok(Self { hotkeys })
    }

    /// Write a hotkey group to an output stream.
    pub(crate) fn write_to<W: Write>(&self, output: &mut W) -> io::Result<()> {
        output.write_u32::<LE>(self.hotkeys.len() as u32)?;
        for hotkey in &self.hotkeys {
            hotkey.write_to(output)?;
        }
        Ok(())
    }

    /// Returns an immutable reference to a single hotkey, if that hotkey is
    /// present in this `HotkeyGroup`.
    pub fn hotkey(&self, index: usize) -> Option<&Hotkey> {
        self.hotkeys.get(index)
    }

    /// Get a mutable reference to a single hotkey.
    /// This way, you can edit or replace the mapping.
    pub fn hotkey_mut(&mut self, index: usize) -> Option<&mut Hotkey> {
        self.hotkeys.get_mut(index)
    }

    /// Returns a hotkey group equivalent to this group but with the hotkey
    /// at `index` unbound so that the key is `0` and all modifier keys are
    /// `false`. Returns an error if the index does not exist.
    pub fn unbind(&self, index: usize) -> Result<Self, HotkeyIndexError> {
        self.bind(index, 0, false, false, false)
    }

    /// Returns a hotkey group equivalent to this group but with the hotkey
    /// at `index` bound with the given key and modifier keys.
    /// Returns an error if the index does not exist.
    pub fn bind(&self, index: usize, key: i32, ctrl: bool, alt: bool,
            shift: bool) -> Result<Self, HotkeyIndexError> {
        if index >= self.num_hotkeys() {
            return Err(HotkeyIndexError::new(index, self.num_hotkeys()));
        }
        let hotkeys = self.hotkeys.iter().enumerate().map(|(i, &hk)|
            if i == index { hk.key(key).ctrl(ctrl).alt(alt).shift(shift) }
            else { hk }
        ).collect();
        Ok(Self { hotkeys })
    }

    /// Returns the number of hotkeys in this `HotkeyGroup`.
    /// ```rust
    /// use std::fs::File;
    /// use genie_hki::{HotkeyInfo, HotkeyGroupId};
    /// let mut f = File::open("test/files/aoc1.hki").unwrap();
    /// let info = HotkeyInfo::from(&mut f).expect("failed to read file");
    /// let group = info.group(HotkeyGroupId::Villager).unwrap();
    /// assert_eq!(28, group.num_hotkeys());
    /// ```
    pub fn num_hotkeys(&self) -> usize { self.hotkeys.len() }

    /// Returns an iterator over this group's hotkeys.
    pub fn iter(&self) -> Iter<Hotkey> { self.hotkeys.iter() }
}

impl fmt::Display for HotkeyGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let group_string = if self.hotkeys.is_empty() {
            String::from(" no hotkeys")
        } else {
            let hotkeys: Vec<String> = self.hotkeys.iter()
                .map(|hk| hk.to_string()).collect();
            format!("\n  {}", hotkeys.join("\n  "))
        };
        write!(f, "Group:{}", group_string)
    }
}

impl IntoIterator for HotkeyGroup {
    type Item = Hotkey;
    type IntoIter = std::vec::IntoIter<Hotkey>;
    fn into_iter(self) -> Self::IntoIter { self.hotkeys.into_iter() }
}

/// Represents a HKI file containing hotkey settings.
#[derive(Debug, Clone, PartialEq)]
pub struct HotkeyInfo {
    /// The file version.
    version: f32,
    /// The hotkey groups.
    groups: Vec<HotkeyGroup>,
}

impl HotkeyInfo {
    /// Read a hotkey info structure from an uncompressed stream.
    fn from_uncompressed<R: Read>(input: &mut R) -> io::Result<Self> {
        let version = input.read_f32::<LE>()?;
        let num_groups = input.read_u32::<LE>()?;
        let mut groups = Vec::with_capacity(num_groups as usize);
        for _ in 0..num_groups {
            groups.push(HotkeyGroup::from(input)?);
        }

        Ok(Self { version, groups })
    }

    /// Read a hotkey info file.
    pub fn from<R: Read>(input: &mut R) -> io::Result<Self> {
        let mut input = DeflateDecoder::new(input);
        Self::from_uncompressed(&mut input)
    }

    /// Write a hotkey info structure to an uncompressed stream.
    fn write_to_uncompressed<W: Write>(&self, output: &mut W)
            -> io::Result<()> {
        output.write_f32::<LE>(self.version)?;
        output.write_u32::<LE>(self.groups.len() as u32)?;
        for group in &self.groups {
            group.write_to(output)?;
        }
        Ok(())
    }

    /// Write a hotkey info file.
    pub fn write_to<W: Write>(&self, output: &mut W) -> io::Result<()> {
        let mut output = DeflateEncoder::new(output, Compression::default());
        self.write_to_uncompressed(&mut output)
    }

    /// Get the file version.
    ///
    /// ```rust
    /// use std::fs::File;
    /// use genie_hki::HotkeyInfo;
    /// let mut f = File::open("test/files/aoc1.hki").unwrap();
    /// let info = HotkeyInfo::from(&mut f).expect("failed to read file");
    /// assert_eq!(info.version(), 1.0);
    ///
    /// let mut f = File::open("test/files/hd0.hki").unwrap();
    /// let info = HotkeyInfo::from(&mut f).expect("failed to read file");
    /// assert_eq!(info.version(), 3.0);
    /// ```
    pub fn version(&self) -> f32 {
        self.version
    }

    /// Get a hotkey group. Groups may not exist in every file.
    ///
    /// ```rust
    /// use std::fs::File;
    /// use genie_hki::{HotkeyInfo, HotkeyGroupId};
    /// let mut f = File::open("test/files/aoc1.hki").unwrap();
    /// let info = HotkeyInfo::from(&mut f).expect("failed to read file");
    /// assert!(info.group(HotkeyGroupId::Villager).is_some());
    /// assert!(info.group(HotkeyGroupId::Mill).is_none());
    /// ```
    pub fn group(&self, group_id: HotkeyGroupId) -> Option<&HotkeyGroup> {
        self.group_raw(group_id as usize)
    }

    /// Returns an immutable reference to a hotkey group, if that group exists.
    fn group_raw(&self, group_id: usize) -> Option<&HotkeyGroup> {
        self.groups.get(group_id)
    }

    /// Returns a mutable reference to a hotkey group, if that group exists.
    pub fn group_mut(&mut self, group_id: HotkeyGroupId)
            -> Option<&mut HotkeyGroup> {
        self.group_mut_raw(group_id as usize)
    }

    /// Returns a mutable reference to a hotkey group, if that group exists.
    fn group_mut_raw(&mut self, group_id: usize) -> Option<&mut HotkeyGroup> {
        self.groups.get_mut(group_id)
    }

    /// Returns the number of hotkey groups in this info's hotkey file.
    /// ```rust
    /// use std::fs::File;
    /// use genie_hki::{HotkeyInfo, HotkeyGroupId};
    ///
    /// let mut f = File::open("test/files/aoc1.hki").unwrap();
    /// let info = HotkeyInfo::from(&mut f).expect("failed to read file");
    /// assert_eq!(14, info.num_groups());
    ///
    /// let mut f = File::open("test/files/wk.hki").unwrap();
    /// let info = HotkeyInfo::from(&mut f).expect("failed to read file");
    /// assert_eq!(15, info.num_groups());
    /// ```
    pub fn num_groups(&self) -> usize { self.groups.len() }

    /// Returns an iterator over the hotkey groups present in this info's hotkey
    /// file.
    pub fn iter(&self) -> Iter<HotkeyGroup> { self.groups.iter() }

    /// Returns a `HotkeyInfo` struct equivalent to this `HotkeyInfo`, but with
    /// the key at index `key_index` of the group given by `group_index`
    /// unbound. Returns an error if either index does not exist.
    pub fn unbind_key(&self, group_index: HotkeyGroupId, key_index: usize)
            -> Result<Self, IndexError> {
        self.bind_key(group_index, key_index, 0, false, false, false)
    }

    /// Returns a `HotkeyInfo` struct equivalent to this `HotkeyInfo`, but with
    /// the key at index `key_index` of the group given by `group_index`
    /// unbound. Returns an error if either index does not exist.
    pub fn unbind_key_index(&self, group_index: usize, key_index: usize)
            -> Result<Self, IndexError> {
        self.bind_key_index(group_index, key_index, 0, false, false, false)
    }

    /// Returns a `HotkeyInfo` struct equivalent to this `HotkeyInfo`, but with
    /// the key at index `key_index` of the group given by `group_index`
    /// bound with the given key and key modifiers. Returns an error if either
    /// index does not exist.
    pub fn bind_key(&self, group_index: HotkeyGroupId, key_index: usize,
            key: i32, ctrl: bool, alt: bool, shift: bool)
            -> Result<Self, IndexError> {
        self.bind_key_index(group_index as usize, key_index, key,
                            ctrl, alt, shift)
    }

    /// Returns a `HotkeyInfo` struct equivalent to this `HotkeyInfo`, but with
    /// the key at index `key_index` of the group given by `group_index`
    /// bound with the given key and key modifiers. Returns an error if either
    /// index does not exist.
    pub fn bind_key_index(&self, group_index: usize, key_index: usize, key: i32,
            ctrl: bool, alt: bool, shift: bool) -> Result<Self, IndexError> {
        if group_index >= self.num_groups() {
            return Err(IndexError::GroupIndex(GroupIndexError::new(
                group_index, self.num_groups())));
        }
        let mut groups = Vec::with_capacity(self.num_groups());
        for (i, grp) in self.groups.iter().enumerate() {
            let append = if i == group_index {
                grp.bind(key_index, key, ctrl, alt, shift)?
            } else {
                grp.clone()
            };
            groups.push(append);
        }
        Ok(Self { groups, ..*self })
    }
}

impl fmt::Display for HotkeyInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let group_string = if self.groups.is_empty() { String::from("") } else {
            let groups: Vec<String> = self.groups.iter()
                .map(|grp| grp.to_string()).collect();
            format!("\n{}", groups.join("\n"))
        };
        write!(f, "Version: {}{}", self.version, group_string)
    }
}

impl IntoIterator for HotkeyInfo {
    type Item = HotkeyGroup;
    type IntoIter = std::vec::IntoIter<HotkeyGroup>;
    fn into_iter(self) -> Self::IntoIter { self.groups.into_iter() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn aoc1() {
        let mut f = File::open("test/files/aoc1.hki").unwrap();
        HotkeyInfo::from(&mut f).expect("failed to read file");
    }

    #[test]
    fn aoc2() {
        let mut f = File::open("test/files/aoc2.hki").unwrap();
        HotkeyInfo::from(&mut f).expect("failed to read file");
    }

    #[test]
    fn aoc3() {
        let mut f = File::open("test/files/aoc3.hki").unwrap();
        HotkeyInfo::from(&mut f).expect("failed to read file");
    }

    #[test]
    fn hd0() {
        let mut f = File::open("test/files/hd0.hki").unwrap();
        HotkeyInfo::from(&mut f).expect("failed to read file");
    }

    #[test]
    fn hd1() {
        let mut f = File::open("test/files/hd1.hki").unwrap();
        HotkeyInfo::from(&mut f).expect("failed to read file");
    }

    #[test]
    fn wk() {
        let mut f = File::open("test/files/wk.hki").unwrap();
        HotkeyInfo::from(&mut f).expect("failed to read file");
    }

    #[test]
    fn hk_group_iter() {
        let mut f = File::open("test/files/aoc1.hki").unwrap();
        let info = HotkeyInfo::from(&mut f).expect("failed to read file");
        let group = info.group(HotkeyGroupId::UnitCommands).unwrap();
        let mut hotkey_iter = group.iter();
        assert_eq!(19214, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19215, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19224, hotkey_iter.next().unwrap().string_id);
        assert_eq!(-1, hotkey_iter.next().unwrap().string_id);
        assert_eq!(-1, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19216, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19225, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19241, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19242, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19221, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19222, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19012, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19000, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19002, hotkey_iter.next().unwrap().string_id);
        assert_eq!(19220, hotkey_iter.next().unwrap().string_id);
        assert_eq!(None, hotkey_iter.next());
    }

    #[test]
    fn hk_info_iter() {
        let mut f = File::open("test/files/aoc1.hki").unwrap();
        let info = HotkeyInfo::from(&mut f).expect("failed to read file");
        let mut iter = info.iter();
        assert_eq!(info.group(HotkeyGroupId::UnitCommands), iter.next());
        assert_eq!(info.group(HotkeyGroupId::GameCommands), iter.next());
        assert_eq!(info.group(HotkeyGroupId::Scroll), iter.next());
        assert_eq!(info.group(HotkeyGroupId::Villager), iter.next());
        assert_eq!(info.group(HotkeyGroupId::TownCenter), iter.next());
        assert_eq!(info.group(HotkeyGroupId::Dock), iter.next());
        assert_eq!(info.group(HotkeyGroupId::Barracks), iter.next());
        assert_eq!(info.group(HotkeyGroupId::ArcheryRange), iter.next());
        assert_eq!(info.group(HotkeyGroupId::Stable), iter.next());
        assert_eq!(info.group(HotkeyGroupId::SiegeWorkshop), iter.next());
        assert_eq!(info.group(HotkeyGroupId::Monastery), iter.next());
        assert_eq!(info.group(HotkeyGroupId::Market), iter.next());
        assert_eq!(info.group(HotkeyGroupId::MilitaryUnits), iter.next());
        assert_eq!(info.group(HotkeyGroupId::Castle), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn hk_group_unbind() {
        let mut f = File::open("test/files/aoc1.hki").unwrap();
        let info = HotkeyInfo::from(&mut f).expect("failed to read file");
        let group0 = info.group(HotkeyGroupId::UnitCommands).unwrap();
        let group1 = group0.unbind(UnitCommandHotkeys::BuildEconomic as usize)
                           .unwrap();
        assert_eq!(66, group0.hotkey(0).unwrap().key);
        assert_eq!(0, group1.hotkey(0).unwrap().key);
    }

    #[test]
    fn hk_group_bind() {
        let mut f = File::open("test/files/aoc1.hki").unwrap();
        let info = HotkeyInfo::from(&mut f).expect("failed to read file");
        let group0 = info.group(HotkeyGroupId::UnitCommands).unwrap();
        let group1 = group0.bind(UnitCommandHotkeys::BuildEconomic as usize, 65,
                               false, false, false).unwrap();
        assert_eq!(66, group0.hotkey(0).unwrap().key);
        assert_eq!(65, group1.hotkey(0).unwrap().key);
    }

    #[test]
    fn hk_group_bad_index() {
        let mut f = File::open("test/files/aoc1.hki").unwrap();
        let info = HotkeyInfo::from(&mut f).expect("failed to read file");
        let group = info.group(HotkeyGroupId::UnitCommands).unwrap();
        let result = group.unbind(99999);
        match result {
            Ok(_) => panic!("Index should be out of bounds."),
            Err(_) => () // test succeeds
        }
    }

    #[test]
    fn hk_info_unbind() {
        let mut f = File::open("test/files/aoc1.hki").unwrap();
        let info0 = HotkeyInfo::from(&mut f).expect("failed to read file");
        let info1 = info0.unbind_key(HotkeyGroupId::UnitCommands,
            UnitCommandHotkeys::BuildEconomic as usize).unwrap();
        assert_eq!(66, info0.group(HotkeyGroupId::UnitCommands).unwrap()
                            .hotkey(0).unwrap().key);
        assert_eq!(0,  info1.group(HotkeyGroupId::UnitCommands).unwrap()
                            .hotkey(0).unwrap().key);
    }

    #[test]
    fn hk_info_bind() {
        let mut f = File::open("test/files/aoc1.hki").unwrap();
        let info0 = HotkeyInfo::from(&mut f).expect("failed to read file");
        let info1 = info0.bind_key(HotkeyGroupId::UnitCommands,
            UnitCommandHotkeys::BuildEconomic as usize, 65, false, false, false)
            .unwrap();
        assert_eq!(66, info0.group(HotkeyGroupId::UnitCommands).unwrap()
                            .hotkey(0).unwrap().key);
        assert_eq!(65, info1.group(HotkeyGroupId::UnitCommands).unwrap()
                            .hotkey(0).unwrap().key);
    }

    #[test]
    fn hk_info_bad_index_group() {
        let mut f = File::open("test/files/aoc1.hki").unwrap();
        let info = HotkeyInfo::from(&mut f).expect("failed to read file");
        let result = info.unbind_key_index(999999, 0);
        match result {
            Ok(_) => panic!("Index should be out of bounds."),
            Err(_) => () // test succeeds
        }
    }

    #[test]
    fn hk_info_bad_index_hk() {
        let mut f = File::open("test/files/aoc1.hki").unwrap();
        let info = HotkeyInfo::from(&mut f).expect("failed to read file");
        let result = info.unbind_key_index(0, 999999);
        match result {
            Ok(_) => panic!("Index should be out of bounds."),
            Err(_) => () // test succeeds
        }
    }
}
