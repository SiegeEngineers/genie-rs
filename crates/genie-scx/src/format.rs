//! This module contains the data format reading/writing.

#![allow(clippy::cognitive_complexity)]

use crate::ai::AIInfo;
use crate::bitmap::Bitmap;
use crate::header::SCXHeader;
use crate::map::Map;
use crate::player::*;
use crate::triggers::TriggerSystem;
use crate::types::*;
use crate::victory::*;
use crate::{Error, Result, VersionBundle};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use genie_support::{
    f32_eq, read_opt_u32, read_str, write_opt_str, write_str, StringKey, UnitTypeID,
};
use std::convert::{TryFrom, TryInto};
use std::io::{self, Read, Write};

/// An object placed in the scenario.
#[derive(Debug, Clone, Default)]
pub struct ScenarioObject {
    /// Position (x, y, z) of this object.
    pub position: (f32, f32, f32),
    /// This object's unique ID.
    pub id: i32,
    /// The type ID of this object.
    pub object_type: UnitTypeID,
    /// State value.
    pub state: u8,
    /// Radian angle this unit is facing.
    pub angle: f32,
    /// Current animation frame.
    pub frame: i16,
    /// ID of the object this object is garrisoned in, or -1 when not
    /// garrisoned.
    pub garrisoned_in: Option<i32>,
}

impl ScenarioObject {
    /// Read a placed object from an input stream.
    pub fn read_from(mut input: impl Read, version: SCXVersion) -> Result<Self> {
        let position = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        let id = input.read_i32::<LE>()?;
        let object_type = input.read_u16::<LE>()?.into();
        let state = input.read_u8()?;
        let angle = input.read_f32::<LE>()?;
        let frame = if version < SCXVersion(*b"1.15") {
            -1
        } else {
            input.read_i16::<LE>()?
        };
        let garrisoned_in = if version < SCXVersion(*b"1.13") {
            None
        } else {
            Some(input.read_i32::<LE>()?)
        }
        .and_then(|id| match id {
            -1 => None,
            id => Some(id),
        })
        .and_then(|id| match id {
            // 0 means -1 in "recent" versions
            0 if version > SCXVersion(*b"1.12") => None,
            id => Some(id),
        });

        Ok(Self {
            position,
            id,
            object_type,
            state,
            angle,
            frame,
            garrisoned_in,
        })
    }

    /// Write placed object data to an output stream.
    pub fn write_to(&self, mut output: impl Write, version: SCXVersion) -> Result<()> {
        output.write_f32::<LE>(self.position.0)?;
        output.write_f32::<LE>(self.position.1)?;
        output.write_f32::<LE>(self.position.2)?;
        output.write_i32::<LE>(self.id)?;
        output.write_u16::<LE>(self.object_type.into())?;
        output.write_u8(self.state)?;
        output.write_f32::<LE>(self.angle)?;
        if version > SCXVersion(*b"1.14") {
            output.write_i16::<LE>(self.frame)?;
        }
        if version > SCXVersion(*b"1.12") {
            match self.garrisoned_in {
                Some(id) => output.write_i32::<LE>(id)?,
                None => output.write_i32::<LE>(-1)?,
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct PlayerData {
    name: Option<String>,
    name_id: Option<StringKey>,
    /// Resources this player has available at the start of the game.
    pub start_resources: PlayerStartResources,
    /// Settings about the player. Is this an AI or a human? What is their civilization? etc.
    base_properties: PlayerBaseProperties,
    /// The starting age for this player.
    pub start_age: StartingAge,
}

impl PlayerData {
    /// Get the name for the player, if one is set.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| s.as_str())
    }

    /// Set or clear the name for the player.
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }

    /// Get the string ID name for the player, if one is set.
    pub fn name_id(&self) -> Option<StringKey> {
        // clone is cheap because this is always a numeric string key.
        self.name_id.clone()
    }

    /// Set or clear the string ID name for the player.
    ///
    /// # Panics
    /// This function panics if an `id` with a named `StringKey` is provided. Only numeric
    /// `StringKey`s are allowed in scenario files.
    pub fn set_name_id(&mut self, id: Option<StringKey>) {
        assert!(id.is_none() || id.as_ref().unwrap().is_numeric(), "only numeric `StringKey`s can be used in scenarios");
        self.name_id = id;
    }

    /// Get the civilization ID this player would play as.
    pub fn civilization(&self) -> i32 {
        self.base_properties.civilization
    }

    /// Set the civilization ID this player would play as.
    pub fn set_civilization(&mut self, id: i32) {
        self.base_properties.civilization = id;
    }

    /// Is this player slot active for this scenario? If not, it will not be used by the game.
    pub fn is_active(&self) -> bool {
        self.base_properties.active != 0
    }
}

/// Embeddable scenario data. This includes all scenario settings, but not map data, triggers, and
/// placed objects.
///
/// The game saves this structure in scenario files, and also in saved and recorded game files.
#[derive(Debug, Clone)]
pub struct ScenarioData {
    /// Data version.
    version: f32,
    player_data: [PlayerData; 16],
    victory_conquest: bool,
    /// File name of this scenario.
    name: String,
    description_string_table: Option<StringKey>,
    hints_string_table: Option<StringKey>,
    win_message_string_table: Option<StringKey>,
    loss_message_string_table: Option<StringKey>,
    history_string_table: Option<StringKey>,
    scout_string_table: Option<StringKey>,
    description: Option<String>,
    hints: Option<String>,
    win_message: Option<String>,
    loss_message: Option<String>,
    history: Option<String>,
    scout: Option<String>,
    pregame_cinematic: Option<String>,
    victory_cinematic: Option<String>,
    loss_cinematic: Option<String>,
    mission_bmp: Option<String>,
    player_build_lists: [Option<String>; 16],
    player_city_plans: [Option<String>; 16],
    player_ai_rules: [Option<String>; 16],
    player_files: [PlayerFiles; 16],
    ai_rules_types: [i8; 16],
    /// Victory settings.
    pub victory: VictoryInfo,
    /// Whether all victory conditions need to be met for victory to occur.
    victory_all_flag: bool,
    /// Type of victory condition to use in multiplayer games.
    mp_victory_type: i32,
    /// Required score to attain multiplayer victory.
    victory_score: i32,
    /// Time at which the highest-scoring player will win the multiplayer match.
    victory_time: i32,
    /// Initial diplomacy stances between players.
    diplomacy: [[DiplomaticStance; 16]; 16],
    legacy_victory_info: [[LegacyVictoryInfo; 12]; 16],
    /// Whether Allied Victory is enabled for each player.
    allied_victory: [i32; 16],
    teams_locked: bool,
    can_change_teams: bool,
    random_start_locations: bool,
    max_teams: u8,
    /// Number of disabled techs per player.
    ///
    /// TODO only use `disabled_techs` for this
    num_disabled_techs: [i32; 16],
    /// Disabled tech IDs per player.
    disabled_techs: [Vec<i32>; 16],
    /// Number of disabled units per player.
    ///
    /// TODO only use `disabled_units` for this
    num_disabled_units: [i32; 16],
    /// Disabled unit IDs per player.
    disabled_units: [Vec<i32>; 16],
    /// Number of disabled buildings per player.
    ///
    /// TODO only use `disabled_buildings` for this
    num_disabled_buildings: [i32; 16],
    /// Disabled building IDs per player.
    disabled_buildings: [Vec<i32>; 16],
    /// (What exactly?)
    ///
    /// According to [AoE2ScenarioParser][].
    /// [AoE2ScenarioParser]: https://github.com/KSneijders/AoE2ScenarioParser/blob/8e3abd422164961aa5c7857350475088790804f8/AoE2ScenarioParser/pieces/options.py
    combat_mode: i32,
    /// (What exactly?)
    ///
    /// According to [AoE2ScenarioParser][].
    /// [AoE2ScenarioParser]: https://github.com/KSneijders/AoE2ScenarioParser/blob/8e3abd422164961aa5c7857350475088790804f8/AoE2ScenarioParser/pieces/options.py
    naval_mode: i32,
    /// Whether "All Techs" is enabled.
    all_techs: bool,
    /// The initial camera location.
    view: (i32, i32),
    /// The map type.
    map_type: Option<i32>,
    base_priorities: [i8; 16],
    /// The water definition type used. (DE2 and up)
    water_definition: Option<String>,
    /// The colour mood used. (DE2 and up)
    color_mood: Option<String>,
    /// Is collide-and-correct pathing enabled?
    ///
    /// Only supported for DE2 and up; defaults to `false` in earlier versions.
    collide_and_correct: bool,
    /// Is villager force drop enabled?
    ///
    /// Only supported for DE2 and up; defaults to `false` in earlier versions.
    villager_force_drop: bool,
}

impl Default for ScenarioData {
    fn default() -> Self {
        Self {
            version: 1.22,
            player_data: Default::default(),
            victory_conquest: false,
            name: String::new(),
            description_string_table: None,
            hints_string_table: None,
            win_message_string_table: None,
            loss_message_string_table: None,
            history_string_table: None,
            scout_string_table: None,
            description: None,
            hints: None,
            win_message: None,
            loss_message: None,
            history: None,
            scout: None,
            pregame_cinematic: None,
            victory_cinematic: None,
            loss_cinematic: None,
            mission_bmp: None,
            player_build_lists: Default::default(),
            player_city_plans: Default::default(),
            player_ai_rules: Default::default(),
            player_files: Default::default(),
            ai_rules_types: Default::default(),
            victory: VictoryInfo::default(),
            victory_all_flag: true,
            mp_victory_type: 4,
            victory_score: 900,
            victory_time: 9000,
            diplomacy: [[DiplomaticStance::Enemy; 16]; 16],
            legacy_victory_info: Default::default(),
            allied_victory: Default::default(),
            teams_locked: false,
            can_change_teams: false,
            random_start_locations: false,
            max_teams: 4,
            num_disabled_techs: Default::default(),
            disabled_techs: Default::default(),
            num_disabled_units: Default::default(),
            disabled_units: Default::default(),
            num_disabled_buildings: Default::default(),
            disabled_buildings: Default::default(),
            combat_mode: 0,
            naval_mode: 0,
            all_techs: false,
            view: (-1, -1),
            map_type: None,
            base_priorities: Default::default(),
            water_definition: None,
            color_mood: None,
            collide_and_correct: false,
            villager_force_drop: false,
        }
    }
}

impl ScenarioData {
    #[deprecated = "Use ScenarioData::read_from instead"]
    #[doc(hidden)]
    pub fn from(input: impl Read) -> Result<Self> {
        Self::read_from(input)
    }

    /// Read scenario data from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut scen = Self::default();
        let version = input.read_f32::<LE>()?;
        log::debug!("RGEScen version {}", version);
        scen.version = version;
        let player_data = &mut scen.player_data;

        // Moved around in 1.13
        if version > 1.13 {
            for player in player_data.iter_mut() {
                player.name = read_str(&mut input, 256)?;
            }
        }

        if version > 1.16 {
            for player in player_data.iter_mut() {
                player.name_id = read_opt_u32(&mut input)?;
            }
        }

        if version > 1.13 {
            for player in player_data.iter_mut() {
                player.base_properties.active = input.read_i32::<LE>()?;
                player.base_properties.player_type = input.read_i32::<LE>()?;
                player.base_properties.civilization = input.read_i32::<LE>()?;
                player.base_properties.posture = input.read_i32::<LE>()?;
            }
        }

        if version >= 1.07 {
            scen.victory_conquest = input.read_u8()? != 0;
        }

        {
            let _timeline_count = input.read_i16::<LE>()?;
            let _timeline_available = input.read_i16::<LE>()?;
            let _old_time = input.read_f32::<LE>()?;
            dbg!(_timeline_count, _timeline_available, _old_time);
            assert_eq!(_timeline_count, 0, "Unexpected RGE_Timeline");
            // assert_eq!(_timeline_available, 0, "Unexpected RGE_Timeline");
        }

        if version >= 1.28 {
            let _civ_lock = &mut [0; 16];
            input.read_u32_into::<LE>(_civ_lock)?;
        }

        scen.name = {
            // File name may be empty for embedded scenario data inside recorded games.
            let len = input.read_i16::<LE>()? as usize;
            read_str(&mut input, len)?.unwrap_or_default()
        };

        if version >= 1.16 {
            scen.description_string_table = read_opt_u32(&mut input)?;
            scen.hints_string_table = read_opt_u32(&mut input)?;
            scen.win_message_string_table = read_opt_u32(&mut input)?;
            scen.loss_message_string_table = read_opt_u32(&mut input)?;
            scen.history_string_table = read_opt_u32(&mut input)?;
        }
        if version >= 1.22 {
            scen.scout_string_table = read_opt_u32(&mut input)?;
        }

        scen.description = {
            let len = input.read_i16::<LE>()? as usize;
            read_str(&mut input, len)?
        };

        if version >= 1.11 {
            scen.hints = {
                let len = input.read_i16::<LE>()? as usize;
                read_str(&mut input, len)?
            };
            scen.win_message = {
                let len = input.read_i16::<LE>()? as usize;
                read_str(&mut input, len)?
            };
            scen.loss_message = {
                let len = input.read_i16::<LE>()? as usize;
                read_str(&mut input, len)?
            };
            scen.history = {
                let len = input.read_i16::<LE>()? as usize;
                read_str(&mut input, len)?
            };
        }
        if version >= 1.22 {
            scen.scout = {
                let len = input.read_i16::<LE>()? as usize;
                read_str(&mut input, len)?
            };
        }

        if version < 1.03 {
            // skip some stuff
        }

        let len = input.read_i16::<LE>()? as usize;
        scen.pregame_cinematic = read_str(&mut input, len)?;
        let len = input.read_i16::<LE>()? as usize;
        scen.victory_cinematic = read_str(&mut input, len)?;
        let len = input.read_i16::<LE>()? as usize;
        scen.loss_cinematic = read_str(&mut input, len)?;

        scen.mission_bmp = if version >= 1.09 {
            let len = input.read_i16::<LE>()? as usize;
            read_str(&mut input, len)?
        } else {
            None
        };

        let _mission_picture = if version >= 1.10 {
            Bitmap::read_from(&mut input)?
        } else {
            None
        };

        for build_list in scen.player_build_lists.iter_mut() {
            let len = input.read_u16::<LE>()? as usize;
            *build_list = read_str(&mut input, len)?;
        }

        for city_plan in scen.player_city_plans.iter_mut() {
            let len = input.read_u16::<LE>()? as usize;
            *city_plan = read_str(&mut input, len)?;
        }

        if version >= 1.08 {
            for ai_rules in scen.player_ai_rules.iter_mut() {
                let len = input.read_u16::<LE>()? as usize;
                *ai_rules = read_str(&mut input, len)?;
            }
        }

        for files in scen.player_files.iter_mut() {
            let build_list_length = input.read_i32::<LE>()? as usize;
            let city_plan_length = input.read_i32::<LE>()? as usize;
            let ai_rules_length = if version >= 1.08 {
                input.read_i32::<LE>()? as usize
            } else {
                0
            };

            files.build_list = read_str(&mut input, build_list_length)?;
            files.city_plan = read_str(&mut input, city_plan_length)?;
            files.ai_rules = read_str(&mut input, ai_rules_length)?;
        }

        if version >= 1.20 {
            for rule_type in scen.ai_rules_types.iter_mut() {
                *rule_type = input.read_i8()?;
            }
        }

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        // Moved around in 1.13
        if version <= 1.13 {
            for player in player_data.iter_mut() {
                player.name = read_str(&mut input, 256)?;
            }

            for player in player_data.iter_mut() {
                player.base_properties.active = input.read_i32::<LE>()?;
                player.start_resources = PlayerStartResources::read_from(&mut input, version)?;
                player.base_properties.player_type = input.read_i32::<LE>()?;
                player.base_properties.civilization = input.read_i32::<LE>()?;
                player.base_properties.posture = input.read_i32::<LE>()?;
            }
        } else {
            for player in player_data.iter_mut() {
                player.start_resources = PlayerStartResources::read_from(&mut input, version)?;
            }
        }

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        scen.victory = VictoryInfo::read_from(&mut input)?;
        scen.victory_all_flag = input.read_i32::<LE>()? != 0;

        scen.mp_victory_type = if version >= 1.13 {
            input.read_i32::<LE>()?
        } else {
            4
        };
        scen.victory_score = if version >= 1.13 {
            input.read_i32::<LE>()?
        } else {
            900
        };
        scen.victory_time = if version >= 1.13 {
            input.read_i32::<LE>()?
        } else {
            9000
        };

        log::debug!(
            "Victory values: {} {} {}",
            scen.mp_victory_type,
            scen.victory_score,
            scen.victory_time
        );

        for player_diplomacy in scen.diplomacy.iter_mut() {
            for stance in player_diplomacy.iter_mut() {
                *stance = DiplomaticStance::try_from(input.read_i32::<LE>()?)?;
            }
        }

        for list in scen.legacy_victory_info.iter_mut() {
            for victory_info in list.iter_mut() {
                *victory_info = LegacyVictoryInfo::read_from(&mut input)?;
            }
        }

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        for setting in scen.allied_victory.iter_mut() {
            *setting = input.read_i32::<LE>()?;
        }

        if version >= 1.24 {
            scen.teams_locked = input.read_i8()? != 0;
            scen.can_change_teams = input.read_i8()? != 0;
            scen.random_start_locations = input.read_i8()? != 0;
            scen.max_teams = input.read_u8()?;
        } else if f32_eq!(version, 1.23) {
            scen.teams_locked = input.read_i32::<LE>()? != 0;
        }

        if version >= 1.28 {
            // Definitive Edition only stores the exact number of disabled techs/units/buildings.
            input.read_i32_into::<LE>(&mut scen.num_disabled_techs)?;
            for (player_disabled_techs, &num) in scen
                .disabled_techs
                .iter_mut()
                .zip(scen.num_disabled_techs.iter())
            {
                *player_disabled_techs = vec![0; num as usize];
                input.read_i32_into::<LE>(player_disabled_techs)?;
            }

            input.read_i32_into::<LE>(&mut scen.num_disabled_units)?;
            for (player_disabled_units, &num) in scen
                .disabled_units
                .iter_mut()
                .zip(scen.num_disabled_units.iter())
            {
                *player_disabled_units = vec![0; num as usize];
                input.read_i32_into::<LE>(player_disabled_units)?;
            }

            input.read_i32_into::<LE>(&mut scen.num_disabled_buildings)?;
            for (player_disabled_buildings, &num) in scen
                .disabled_buildings
                .iter_mut()
                .zip(scen.num_disabled_buildings.iter())
            {
                *player_disabled_buildings = vec![0; num as usize];
                input.read_i32_into::<LE>(player_disabled_buildings)?;
            }
        } else if version >= 1.18 {
            // AoC and friends store up to 20 or 30 of each.
            input.read_i32_into::<LE>(&mut scen.num_disabled_techs)?;
            for player_disabled_techs in scen.disabled_techs.iter_mut() {
                *player_disabled_techs = vec![0; 30];
                input.read_i32_into::<LE>(player_disabled_techs)?;
            }

            input.read_i32_into::<LE>(&mut scen.num_disabled_units)?;
            for player_disabled_units in scen.disabled_units.iter_mut() {
                *player_disabled_units = vec![0; 30];
                input.read_i32_into::<LE>(player_disabled_units)?;
            }

            input.read_i32_into::<LE>(&mut scen.num_disabled_buildings)?;
            let max_disabled_buildings = if version >= 1.25 { 30 } else { 20 };
            for player_disabled_buildings in scen.disabled_buildings.iter_mut() {
                *player_disabled_buildings = vec![0; max_disabled_buildings];
                input.read_i32_into::<LE>(player_disabled_buildings)?;
            }
        } else if version > 1.03 {
            // Old scenarios only allowed disabling up to 20 techs per player.
            for i in 0..16 {
                let player_disabled_techs = &mut scen.disabled_techs[i];
                *player_disabled_techs = vec![0; 20];
                input.read_i32_into::<LE>(player_disabled_techs)?;
                // The number of disabled techs wasn't stored either, so we need to guess it!
                scen.num_disabled_techs[i] = player_disabled_techs
                    .iter()
                    .position(|val| *val <= 0)
                    .map(|index| (index as i32) + 1)
                    .unwrap_or(0);
            }
        } else {
            // <= 1.03 did not support disabling anything
        }

        if version > 1.04 {
            scen.combat_mode = input.read_i32::<LE>()?;
        }
        if version >= 1.12 {
            scen.naval_mode = input.read_i32::<LE>()?;
            scen.all_techs = input.read_i32::<LE>()? != 0;
        }

        if version > 1.05 {
            for player in player_data.iter_mut() {
                player.start_age = StartingAge::try_from(input.read_i32::<LE>()?, version)?;
            }
        }

        log::debug!(
            "starting ages: {:?}",
            player_data
                .iter()
                .map(|player| player.start_age)
                .collect::<Vec<_>>()
        );

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        scen.view = if version >= 1.19 {
            (input.read_i32::<LE>()?, input.read_i32::<LE>()?)
        } else {
            (-1, -1)
        };

        scen.map_type = if version >= 1.21 {
            match input.read_i32::<LE>()? {
                // HD Edition uses -2 instead of -1?
                -2 | -1 => None,
                id => Some(
                    id.try_into()
                        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?,
                ),
            }
        } else {
            None
        };

        if version >= 1.24 {
            input.read_i8_into(&mut scen.base_priorities)?;
        }

        if version >= 1.35 {
            // Duplicated here from TriggerSystem … we can discard it because the TriggerSystem
            // will read the same number later.
            let _trigger_count = input.read_u32::<LE>()?;
        }
        if version >= 1.30 {
            let _str_signature = input.read_u16::<LE>()?;
            scen.water_definition = {
                let len = input.read_u16::<LE>()?;
                read_str(&mut input, len as usize)?
            };
        }

        if version >= 1.32 {
            let _str_signature = input.read_u16::<LE>()?;
            scen.color_mood = {
                let len = input.read_u16::<LE>()?;
                read_str(&mut input, len as usize)?
            };
        }
        if version >= 1.36 {
            scen.collide_and_correct = input.read_u8()? != 0;
        }
        if version >= 1.37 {
            scen.villager_force_drop = input.read_u8()? != 0;
        }

        Ok(scen)
    }

    /// Write scenario data to an output stream.
    pub fn write_to(&self, mut output: impl Write, version: f32, num_triggers: u32) -> Result<()> {
        output.write_f32::<LE>(version)?;

        if version > 1.13 {
            for player in &self.player_data {
                let mut padded_bytes = Vec::with_capacity(256);
                if let Some(ref name) = player.name {
                    let name_bytes = name.as_bytes();
                    padded_bytes.write_all(name_bytes)?;
                }
                padded_bytes.extend(vec![0; 256 - padded_bytes.len()]);
                output.write_all(&padded_bytes)?;
            }
        }

        if version > 1.16 {
            for player in &self.player_data {
                write_opt_string_key(&mut output, &player.name_id)?;
            }
        }

        if version > 1.13 {
            for PlayerData {
                base_properties, ..
            } in &self.player_data
            {
                output.write_i32::<LE>(base_properties.active)?;
                output.write_i32::<LE>(base_properties.player_type)?;
                output.write_i32::<LE>(base_properties.civilization)?;
                output.write_i32::<LE>(base_properties.posture)?;
            }
        }

        if version >= 1.07 {
            output.write_u8(if self.victory_conquest { 1 } else { 0 })?;
        }

        // RGE_Timeline
        output.write_i16::<LE>(0)?;
        output.write_i16::<LE>(0)?;
        output.write_f32::<LE>(-1.0)?;

        if version >= 1.28 {
            // Civ Lock data
            for _ in 0..16 {
                output.write_u32::<LE>(0)?;
            }
        }

        write_str(&mut output, &self.name)?;

        if version >= 1.16 {
            write_opt_string_key(&mut output, &self.description_string_table)?;
            write_opt_string_key(&mut output, &self.hints_string_table)?;
            write_opt_string_key(&mut output, &self.win_message_string_table)?;
            write_opt_string_key(&mut output, &self.loss_message_string_table)?;
            write_opt_string_key(&mut output, &self.history_string_table)?;
        }
        if version >= 1.22 {
            write_opt_string_key(&mut output, &self.scout_string_table)?;
        }

        write_opt_str(&mut output, &self.description)?;
        if version >= 1.11 {
            write_opt_str(&mut output, &self.hints)?;
            write_opt_str(&mut output, &self.win_message)?;
            write_opt_str(&mut output, &self.loss_message)?;
            write_opt_str(&mut output, &self.history)?;
        }
        if version >= 1.22 {
            write_opt_str(&mut output, &self.scout)?;
        }

        write_opt_str(&mut output, &self.pregame_cinematic)?;
        write_opt_str(&mut output, &self.victory_cinematic)?;
        write_opt_str(&mut output, &self.loss_cinematic)?;
        if version >= 1.09 {
            // mission_bmp
            write_opt_str(&mut output, &None)?;
        }

        if version >= 1.10 {
            // mission_picture
            output.write_u32::<LE>(0)?;
            output.write_u32::<LE>(0)?;
            output.write_u32::<LE>(0)?;
            output.write_u16::<LE>(1)?;
        }

        for build_list in &self.player_build_lists {
            write_opt_str(&mut output, build_list)?;
        }

        for city_plan in &self.player_city_plans {
            write_opt_str(&mut output, city_plan)?;
        }

        if version >= 1.08 {
            for ai_rules in &self.player_ai_rules {
                write_opt_str(&mut output, ai_rules)?;
            }
        }

        for files in &self.player_files {
            if let Some(build_list) = &files.build_list {
                output.write_u32::<LE>(build_list.len() as u32)?;
            } else {
                output.write_u32::<LE>(0)?;
            }
            if let Some(city_plan) = &files.city_plan {
                output.write_u32::<LE>(city_plan.len() as u32)?;
            } else {
                output.write_u32::<LE>(0)?;
            }
            if version >= 1.08 {
                if let Some(ai_rules) = &files.ai_rules {
                    output.write_u32::<LE>(ai_rules.len() as u32)?;
                } else {
                    output.write_u32::<LE>(0)?;
                }
            }
            if let Some(build_list) = &files.build_list {
                output.write_all(build_list.as_bytes())?;
            }
            if let Some(city_plan) = &files.city_plan {
                output.write_all(city_plan.as_bytes())?;
            }
            if version >= 1.08 {
                if let Some(ai_rules) = &files.ai_rules {
                    output.write_all(ai_rules.as_bytes())?;
                }
            }
        }

        if version >= 1.20 {
            for ai_rules_type in &self.ai_rules_types {
                output.write_i8(*ai_rules_type)?;
            }
        }

        output.write_i32::<LE>(-99)?;

        if version <= 1.13 {
            for player in &self.player_data {
                let mut padded_bytes = Vec::with_capacity(256);
                if let Some(ref name) = player.name {
                    let name_bytes = name.as_bytes();
                    padded_bytes.write_all(name_bytes)?;
                }
                padded_bytes.extend(vec![0; 256 - padded_bytes.len()]);
                output.write_all(&padded_bytes)?;
            }

            for player in &self.player_data {
                output.write_i32::<LE>(player.base_properties.active)?;
                player.start_resources.write_to(&mut output, version)?;
                output.write_i32::<LE>(player.base_properties.player_type)?;
                output.write_i32::<LE>(player.base_properties.civilization)?;
                output.write_i32::<LE>(player.base_properties.posture)?;
            }
        } else {
            for PlayerData {
                start_resources, ..
            } in &self.player_data
            {
                start_resources.write_to(&mut output, version)?;
            }
        }

        if version >= 1.02 {
            output.write_i32::<LE>(-99)?;
        }

        self.victory.write_to(&mut output)?;
        output.write_i32::<LE>(if self.victory_all_flag { 1 } else { 0 })?;

        if version >= 1.13 {
            output.write_i32::<LE>(self.mp_victory_type)?;
            output.write_i32::<LE>(self.victory_score)?;
            output.write_i32::<LE>(self.victory_time)?;
        }

        for player_diplomacy in &self.diplomacy {
            for stance in player_diplomacy {
                output.write_i32::<LE>((*stance).into())?;
            }
        }

        for list in &self.legacy_victory_info {
            for entry in list {
                entry.write_to(&mut output)?;
            }
        }

        if version >= 1.02 {
            output.write_i32::<LE>(-99)?;
        }

        for value in &self.allied_victory {
            output.write_i32::<LE>(*value)?;
        }

        if version >= 1.24 {
            output.write_i8(if self.teams_locked { 1 } else { 0 })?;
            output.write_i8(if self.can_change_teams { 1 } else { 0 })?;
            output.write_i8(if self.random_start_locations { 1 } else { 0 })?;
            output.write_u8(self.max_teams)?;
        } else if f32_eq!(version, 1.23) {
            output.write_i32::<LE>(if self.teams_locked { 1 } else { 0 })?;
        }

        if version >= 1.28 {
            for num in &self.num_disabled_techs {
                output.write_i32::<LE>(*num)?;
            }
            for (player_disabled_techs, &num) in self
                .disabled_techs
                .iter()
                .zip(self.num_disabled_techs.iter())
            {
                for i in 0..num as usize {
                    output.write_i32::<LE>(*player_disabled_techs.get(i).unwrap_or(&-1))?;
                }
            }

            for num in &self.num_disabled_units {
                output.write_i32::<LE>(*num)?;
            }
            for (player_disabled_units, &num) in self
                .disabled_units
                .iter()
                .zip(self.num_disabled_units.iter())
            {
                for i in 0..num as usize {
                    output.write_i32::<LE>(*player_disabled_units.get(i).unwrap_or(&-1))?;
                }
            }

            for num in &self.num_disabled_buildings {
                output.write_i32::<LE>(*num)?;
            }
            for (player_disabled_buildings, &num) in self
                .disabled_buildings
                .iter()
                .zip(self.num_disabled_buildings.iter())
            {
                for i in 0..num as usize {
                    output.write_i32::<LE>(*player_disabled_buildings.get(i).unwrap_or(&-1))?;
                }
            }
        } else if version >= 1.18 {
            let max_disabled_buildings = if version >= 1.25 { 30 } else { 20 };
            let most = *self.num_disabled_buildings.iter().max().unwrap_or(&0);
            if most > max_disabled_buildings {
                return Err(Error::TooManyDisabledBuildingsError(
                    most,
                    max_disabled_buildings,
                ));
            }

            for num in &self.num_disabled_techs {
                output.write_i32::<LE>(*num)?;
            }
            for player_disabled_techs in &self.disabled_techs {
                for i in 0..30 {
                    output.write_i32::<LE>(*player_disabled_techs.get(i).unwrap_or(&-1))?;
                }
            }

            for num in &self.num_disabled_units {
                output.write_i32::<LE>(*num)?;
            }
            for player_disabled_units in &self.disabled_units {
                for i in 0..30 {
                    output.write_i32::<LE>(*player_disabled_units.get(i).unwrap_or(&-1))?;
                }
            }

            for num in &self.num_disabled_buildings {
                output.write_i32::<LE>(*num)?;
            }
            for player_disabled_buildings in &self.disabled_buildings {
                for i in 0..max_disabled_buildings as usize {
                    output.write_i32::<LE>(*player_disabled_buildings.get(i).unwrap_or(&-1))?;
                }
            }
        } else if version > 1.03 {
            let most = *self.num_disabled_techs.iter().max().unwrap_or(&0);
            if most > 20 {
                return Err(Error::TooManyDisabledTechsError(most));
            }
            if self.num_disabled_units.iter().any(|&n| n > 0) {
                return Err(Error::CannotDisableUnitsError);
            }
            if self.num_disabled_buildings.iter().any(|&n| n > 0) {
                return Err(Error::CannotDisableBuildingsError);
            }

            // Old scenarios only allowed disabling up to 20 techs per player.
            for player_disabled_techs in &self.disabled_techs {
                for i in 0..20 {
                    output.write_i32::<LE>(*player_disabled_techs.get(i).unwrap_or(&-1))?;
                }
            }
        } else {
            // <= 1.03 did not support disabling anything
            if self.num_disabled_techs.iter().any(|&n| n > 0) {
                return Err(Error::CannotDisableTechsError);
            }
            if self.num_disabled_units.iter().any(|&n| n > 0) {
                return Err(Error::CannotDisableUnitsError);
            }
            if self.num_disabled_buildings.iter().any(|&n| n > 0) {
                return Err(Error::CannotDisableBuildingsError);
            }
        }

        if version > 1.04 {
            output.write_i32::<LE>(0)?;
        }
        if version >= 1.12 {
            output.write_i32::<LE>(0)?;
            output.write_i32::<LE>(if self.all_techs { 1 } else { 0 })?;
        }

        if version > 1.05 {
            for PlayerData { start_age, .. } in &self.player_data {
                output.write_i32::<LE>(start_age.to_i32(version))?;
            }
        }

        if version >= 1.02 {
            output.write_i32::<LE>(-99)?;
        }

        if version >= 1.19 {
            output.write_i32::<LE>(self.view.0)?;
            output.write_i32::<LE>(self.view.1)?;
        }

        if version >= 1.21 {
            output.write_i32::<LE>(self.map_type.unwrap_or(-1))?;
        }

        if version >= 1.24 {
            for priority in &self.base_priorities {
                output.write_i8(*priority)?;
            }
        }

        if version >= 1.28 {
            output.write_u32::<LE>(num_triggers)?;
            output.write_u16::<LE>(0)?;
            write_opt_str(&mut output, &self.water_definition)?;
        }

        if version >= 1.36 {
            output.write_u8(0)?;
            output.write_u8(0)?;
            write_opt_str(&mut output, &self.color_mood)?;
            output.write_u8(if self.collide_and_correct { 1 } else { 0 })?;
        }
        if version >= 1.37 {
            output.write_u8(if self.villager_force_drop { 1 } else { 0 })?;
        }

        Ok(())
    }

    /// Get the version of the scenario data.
    pub fn version(&self) -> f32 {
        self.version
    }

    /// Get the file name of the scenario. May be empty if this scenario data is embedded inside
    /// some other filetype, like a saved or recorded game.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the description of the scenario, if any.
    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|s| s.as_str())
    }

    /// Access data for the 8 usable players.
    pub fn players(&self) -> &[PlayerData] {
        &self.player_data[0..8]
    }

    /// Get data for a particular player ID.
    pub fn player(&self, id: u8) -> &PlayerData {
        &self.player_data[id as usize]
    }

    /// Iterate over the data for the active players in this scenario.
    pub fn active_players(&self) -> impl Iterator<Item = &PlayerData> {
        self.players().iter().filter(|player| player.is_active())
    }
}

#[derive(Debug, Clone)]
pub struct SCXFormat {
    /// Version of the SCX format.
    pub(crate) version: SCXVersion,
    /// Uncompressed header containing metadata for display.
    pub(crate) header: SCXHeader,
    /// ID for the next-placed/created object.
    pub(crate) next_object_id: i32,
    /// Scenario data.
    pub(crate) data: ScenarioData,
    /// Map data.
    pub(crate) map: Map,
    /// Player data.
    world_players: Vec<WorldPlayerData>,
    /// Objects data.
    pub(crate) player_objects: Vec<Vec<ScenarioObject>>,
    /// Player data.
    scenario_players: Vec<ScenarioPlayerData>,
    /// Triggers (only in AoK and up).
    pub(crate) triggers: Option<TriggerSystem>,
    /// AI information (AoK and up).
    ai_info: Option<AIInfo>,
}

impl SCXFormat {
    /// Extract version bundle information from a parsed SCX file.
    pub fn version(&self) -> VersionBundle {
        VersionBundle {
            format: self.version,
            header: self.header.version,
            data: self.data.version(),
            triggers: self.triggers.as_ref().map(|triggers| triggers.version()),
            map: self.map.version(),
            ..VersionBundle::aoc()
        }
    }

    fn load_inner(version: SCXVersion, player_version: f32, mut input: impl Read) -> Result<Self> {
        let header = SCXHeader::read_from(&mut input, version)?;

        let mut input = DeflateDecoder::new(&mut input);
        let next_object_id = input.read_i32::<LE>()?;

        let data = ScenarioData::read_from(&mut input)?;

        let map = Map::read_from(&mut input)?;

        let num_players = input.read_u32::<LE>()?;
        log::debug!("number of players: {}", num_players);
        let mut world_players = Vec::with_capacity(num_players as usize);
        for _ in 1..num_players {
            world_players.push(WorldPlayerData::read_from(&mut input, player_version)?);
        }

        fn read_scenario_players(
            mut input: impl Read,
            player_version: f32,
        ) -> Result<Vec<ScenarioPlayerData>> {
            let num = input.read_u32::<LE>()?;
            let mut players = Vec::with_capacity(num as usize);
            log::debug!("number of scenario players: {}", num);
            for _ in 1..num {
                players.push(ScenarioPlayerData::read_from(&mut input, player_version)?);
            }
            Ok(players)
        }

        fn read_player_objects(
            mut input: impl Read,
            num_players: u32,
            version: SCXVersion,
        ) -> Result<Vec<Vec<ScenarioObject>>> {
            let mut player_objects = Vec::with_capacity(num_players as usize);
            for _ in 0..num_players {
                let num_objects = input.read_u32::<LE>()?;
                let mut objects = Vec::with_capacity(num_objects as usize);
                log::debug!("number of objects: {}", num_objects);
                for _ in 0..num_objects {
                    objects.push(ScenarioObject::read_from(&mut input, version)?);
                }
                player_objects.push(objects);
            }
            Ok(player_objects)
        }

        // The order is flipped … thanks DE
        let (scenario_players, player_objects) = if version >= SCXVersion(*b"1.36") {
            let players = read_scenario_players(&mut input, player_version)?;
            let objects = read_player_objects(&mut input, num_players, version)?;
            (players, objects)
        } else {
            let objects = read_player_objects(&mut input, num_players, version)?;
            let players = read_scenario_players(&mut input, player_version)?;
            (players, objects)
        };

        let triggers = if version < SCXVersion(*b"1.14") {
            None
        } else {
            Some(TriggerSystem::read_from(&mut input)?)
        };

        let ai_info = if version > SCXVersion(*b"1.17") && version < SCXVersion(*b"2.00") {
            AIInfo::read_from(&mut input)?
        } else {
            None
        };

        Ok(SCXFormat {
            version,
            header,
            next_object_id,
            data,
            map,
            world_players,
            player_objects,
            scenario_players,
            triggers,
            ai_info,
        })
    }

    pub fn load_scenario(mut input: impl Read) -> Result<Self> {
        let mut format_version = [0; 4];
        input.read_exact(&mut format_version)?;
        let format_version = SCXVersion(format_version);
        if let Some(player_version) = format_version.to_player_version() {
            Self::load_inner(format_version, player_version, input)
        } else {
            Err(Error::UnsupportedFormatVersionError(format_version))
        }
    }

    fn write_player_objects(
        &self,
        mut output: impl Write,
        format_version: SCXVersion,
    ) -> Result<()> {
        for objects in &self.player_objects {
            output.write_i32::<LE>(objects.len() as i32)?;
            for object in objects {
                object.write_to(&mut output, format_version)?;
            }
        }
        Ok(())
    }

    fn write_scenario_players(
        &self,
        mut output: impl Write,
        player_version: f32,
        victory_version: f32,
    ) -> Result<()> {
        output.write_i32::<LE>(self.scenario_players.len() as i32 + 1)?;
        for player in &self.scenario_players {
            player.write_to(&mut output, player_version, victory_version)?;
        }
        Ok(())
    }

    pub fn write_to(&self, mut output: impl Write, version: &VersionBundle) -> Result<()> {
        let player_version = match version.format.to_player_version() {
            Some(v) => v,
            None => return Err(Error::UnsupportedFormatVersionError(version.format)),
        };

        output.write_all(version.format.as_bytes())?;
        self.header
            .write_to(&mut output, version.format, version.header)?;

        let mut output = DeflateEncoder::new(output, Compression::default());
        output.write_i32::<LE>(self.next_object_id)?;

        let num_triggers = self
            .triggers
            .as_ref()
            .map(|trigger_system| trigger_system.num_triggers())
            .unwrap_or(0);
        self.data
            .write_to(&mut output, version.data, num_triggers)?;
        self.map.write_to(&mut output, version.map)?;

        output.write_i32::<LE>(self.player_objects.len() as i32)?;
        for player in &self.world_players {
            player.write_to(&mut output, player_version)?;
        }

        if version.format >= SCXVersion(*b"1.36") {
            self.write_scenario_players(&mut output, player_version, version.victory)?;
            self.write_player_objects(&mut output, version.format)?;
        } else {
            self.write_player_objects(&mut output, version.format)?;
            self.write_scenario_players(&mut output, player_version, version.victory)?;
        }

        if version.format > SCXVersion(*b"1.13") {
            let def = TriggerSystem::default();
            let triggers = match self.triggers {
                Some(ref tr) => tr,
                None => &def,
            };
            triggers.write_to(&mut output, version.triggers.unwrap_or(1.6))?;
        }

        if version.format > SCXVersion(*b"1.17") && version.format < SCXVersion(*b"2.00") {
            let def = AIInfo::default();
            let ai_info = match self.ai_info {
                Some(ref ai) => ai,
                None => &def,
            };
            ai_info.write_to(&mut output)?;
        }

        output.finish()?;

        Ok(())
    }

    /// Get the name of the UserPatch mod that was used to create this scenario, if applicable.
    ///
    /// Returns None if no mod was used.
    pub fn mod_name(&self) -> Option<&str> {
        self.data.player_data[9]
            .name
            .as_ref()
            .map(|string| string.as_str())
    }

    /// Hash the scenario, for comparison with other instances.
    ///
    /// This is only available in tests and the implementation is horrifying :)
    #[cfg(test)]
    pub fn hash(&self) -> u64 {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };
        let mut hasher = DefaultHasher::new();
        format!("{:#?}", self).hash(&mut hasher);
        hasher.finish()
    }
}

fn write_opt_string_key(mut output: impl Write, opt_key: &Option<StringKey>) -> Result<()> {
    output.write_u32::<LE>(if let Some(key) = opt_key {
        key.try_into()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?
    } else {
        0xFFFF_FFFF
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SCXFormat;
    use crate::{Result, VersionBundle};
    use std::fs::File;
    use std::io::{Cursor, ErrorKind, Read};

    fn save_and_load(format: &SCXFormat, as_version: VersionBundle) -> Result<SCXFormat> {
        let mut out = vec![];
        format.write_to(&mut out, &as_version)?;

        let mut f = Cursor::new(out);
        let scx = SCXFormat::load_scenario(&mut f)?;
        assert_consumed(f);
        Ok(scx)
    }

    fn assert_consumed(mut input: impl Read) {
        let byte = &mut [0];
        match input.read_exact(byte) {
            Err(err) if err.kind() == ErrorKind::UnexpectedEof => (),
            Err(err) => panic!("{}", err),
            Ok(_) => {
                let mut trailing_data = vec![byte[0]];
                input.read_to_end(&mut trailing_data).unwrap();
                panic!("data left in buffer ({}): {:?}", trailing_data.len(), {
                    trailing_data.truncate(32);
                    trailing_data
                });
            }
        }
    }

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=42
    #[test]
    fn oldest_aoe1_scn_on_aoeheaven() {
        let mut f = File::open("test/scenarios/ The Destruction of Rome.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    #[test]
    fn aoe1_beta_scn_reserialize() {
        let mut f = File::open("test/scenarios/Dawn of a New Age.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let format2 = save_and_load(&format, format.version()).expect("save-and-load failed");

        assert_eq!(
            format.hash(),
            format2.hash(),
            "should produce exactly the same scenario"
        );
    }

    #[test]
    fn aoe1_beta_scn_to_aoc() {
        let mut f = File::open("test/scenarios/Dawn of a New Age.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let format2 = save_and_load(&format, VersionBundle::aoc()).expect("save-and-load failed");

        assert_eq!(
            format2.version(),
            VersionBundle::aoc(),
            "should have converted to AoC versions"
        );
    }

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=1678
    #[test]
    fn aoe1_trial_scn() {
        let mut f = File::open("test/scenarios/Bronze Age Art of War.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=2409
    #[test]
    fn aoe1_ppc_trial_scn() {
        let mut f = File::open("test/scenarios/CEASAR.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=1651
    #[test]
    fn aoe1_scn() {
        let mut f = File::open("test/scenarios/A New Emporer.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=880
    #[test]
    fn aoe1_ror_scx() {
        let mut f = File::open("test/scenarios/Jeremiah Johnson (Update).scx").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    #[test]
    fn aoe1_ror_to_aoc() -> Result<()> {
        let mut f = File::open("test/scenarios/El advenimiento de los hunos_.scx")?;
        let format = SCXFormat::load_scenario(&mut f)?;
        assert_consumed(f);
        let format2 = save_and_load(&format, VersionBundle::aoc())?;

        assert_eq!(
            format2.version(),
            VersionBundle::aoc(),
            "should have converted to AoC versions"
        );

        Ok(())
    }

    /// Source: http://aok.heavengames.com/blacksmith/showfile.php?fileid=1271
    #[test]
    fn oldest_aok_scn_on_aokheaven() {
        let mut f = File::open("test/scenarios/CAMELOT.SCN").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    #[test]
    fn aoc_scx() {
        let mut f = File::open("test/scenarios/Age of Heroes b1-3-5.scx").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    #[test]
    fn hd_aoe2scenario() {
        let mut f = File::open("test/scenarios/Year_of_the_Pig.aoe2scenario").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let format2 = save_and_load(&format, format.version()).expect("save-and-load failed");

        assert_eq!(
            format.hash(),
            format2.hash(),
            "should produce exactly the same scenario"
        );
    }

    #[test]
    fn hd_scx2() {
        let mut f = File::open("test/scenarios/real_world_amazon.scx").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    /// A Definitive Edition scenario.
    ///
    /// (Ignored because it doesn't work yet.)
    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=2708
    #[test]
    #[ignore]
    fn aoe_de_scn() {
        let mut f = File::open("test/scenarios/Corlis.aoescn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    /// A Definitive Edition 2 scenario, SCX format version 1.36.
    ///
    /// Source: https://www.ageofempires.com/mods/details/2015/
    #[test]
    fn aoe_de2_1_36() {
        let mut f = File::open("test/scenarios/Hotkey Trainer Buildings.aoe2scenario").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let format2 = save_and_load(&format, format.version()).expect("save-and-load failed");
        assert_eq!(
            format.hash(),
            format2.hash(),
            "should produce exactly the same scenario"
        );
    }

    /// A Definitive Edition 2 scenario, based on the included AIImprovementsBucket10Test file,
    /// saved as a 1.37 format version with some layered terrain in the corners.
    #[test]
    fn aoe_de2_1_37() {
        let mut f = File::open("test/scenarios/layertest.aoe2scenario").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_consumed(f);
        let format2 = save_and_load(&format, format.version()).expect("save-and-load failed");
        assert_eq!(
            format.hash(),
            format2.hash(),
            "should produce exactly the same scenario"
        );
    }
}
