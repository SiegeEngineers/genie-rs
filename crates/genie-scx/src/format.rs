//! This module contains the data format reading/writing.

#![allow(clippy::cognitive_complexity)]

use crate::{
    ai::AIInfo, bitmap::Bitmap, header::SCXHeader, map::Map, player::*, triggers::TriggerSystem,
    types::*, util::*, victory::*, Error, Result, VersionBundle,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use genie_support::{cmp_float, read_opt_u32, MapInto, StringKey, UnitTypeID};
use std::{
    cmp::Ordering,
    convert::{TryFrom, TryInto},
    io::{self, Read, Write},
};

fn cmp_scx_version(a: SCXVersion, b: SCXVersion) -> Ordering {
    match a[0].cmp(&b[0]) {
        Ordering::Equal => {}
        ord => return ord,
    }
    match a[2].cmp(&b[2]) {
        Ordering::Equal => {}
        ord => return ord,
    }
    a[3].cmp(&b[3])
}

// pub enum LostInformation {
//     DisabledTechs(i32, i32),
//     DisabledUnits(i32, i32),
//     DisabledBuildings(i32, i32),
//     MapType,
// }

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
    pub fn from<R: Read>(input: &mut R, version: SCXVersion) -> Result<Self> {
        let position = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        let id = input.read_i32::<LE>()?;
        let object_type = input.read_u16::<LE>()?.into();
        let state = input.read_u8()?;
        let angle = input.read_f32::<LE>()?;
        let frame = if cmp_scx_version(version, *b"1.15") == Ordering::Less {
            -1
        } else {
            input.read_i16::<LE>()?
        };
        let garrisoned_in = if cmp_scx_version(version, *b"1.13") == Ordering::Less {
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
            0 if cmp_scx_version(version, *b"1.12") == Ordering::Greater => None,
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

    pub fn write_to<W: Write>(&self, output: &mut W, version: SCXVersion) -> Result<()> {
        output.write_f32::<LE>(self.position.0)?;
        output.write_f32::<LE>(self.position.1)?;
        output.write_f32::<LE>(self.position.2)?;
        output.write_i32::<LE>(self.id)?;
        output.write_u16::<LE>(self.object_type.into())?;
        output.write_u8(self.state)?;
        output.write_f32::<LE>(self.angle)?;
        if cmp_scx_version(version, *b"1.14") == Ordering::Greater {
            output.write_i16::<LE>(self.frame)?;
        }
        if cmp_scx_version(version, *b"1.12") == Ordering::Greater {
            match self.garrisoned_in {
                Some(id) => output.write_i32::<LE>(id)?,
                None => output.write_i32::<LE>(-1)?,
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RGEScen {
    /// Data version.
    pub(crate) version: f32,
    /// Names for each player.
    player_names: Vec<Option<String>>,
    /// Name IDs for each player.
    player_string_table: Vec<Option<StringKey>>,
    player_base_properties: Vec<PlayerBaseProperties>,
    victory_conquest: bool,
    /// File name of this scenario.
    pub(crate) name: String,
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
    player_build_lists: Vec<Option<String>>,
    player_city_plans: Vec<Option<String>>,
    player_ai_rules: Vec<Option<String>>,
    player_files: Vec<PlayerFiles>,
    ai_rules_types: Vec<i8>,
}

impl RGEScen {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let version = input.read_f32::<LE>()?;
        let mut player_names = vec![None; 16];
        if version > 1.13 {
            for name in player_names.iter_mut() {
                *name = read_str(input, 256)?;
            }
        }

        let mut player_string_table = vec![None; 16];
        if version > 1.16 {
            for string_id in player_string_table.iter_mut() {
                *string_id = read_opt_u32(input)?.map_into();
            }
        }

        let mut player_base_properties = vec![PlayerBaseProperties::default(); 16];
        if version > 1.13 {
            for properties in player_base_properties.iter_mut() {
                properties.active = input.read_i32::<LE>()?;
                properties.player_type = input.read_i32::<LE>()?;
                properties.civilization = input.read_i32::<LE>()?;
                properties.posture = input.read_i32::<LE>()?;
            }
        }

        let victory_conquest = if version >= 1.07 {
            input.read_u8()? != 0
        } else {
            true
        };

        assert_eq!(input.read_i16::<LE>()?, 0, "Unexpected RGE_Timeline");
        assert_eq!(input.read_i16::<LE>()?, 0, "Unexpected RGE_Timeline");
        assert!([-1.0, 0.0].contains(&input.read_f32::<LE>()?));

        let name_length = input.read_i16::<LE>()? as usize;
        let name = read_str(input, name_length)?.ok_or(Error::MissingFileNameError)?;

        let (
            description_string_table,
            hints_string_table,
            win_message_string_table,
            loss_message_string_table,
            history_string_table,
        ) = if version >= 1.16 {
            (
                read_opt_u32(input)?.map_into(),
                read_opt_u32(input)?.map_into(),
                read_opt_u32(input)?.map_into(),
                read_opt_u32(input)?.map_into(),
                read_opt_u32(input)?.map_into(),
            )
        } else {
            Default::default()
        };

        let scout_string_table = if version >= 1.22 {
            read_opt_u32(input)?.map_into()
        } else {
            Default::default()
        };

        let description_length = input.read_i16::<LE>()? as usize;
        let description = read_str(input, description_length)?;

        let (hints, win_message, loss_message, history) = if version >= 1.11 {
            let hints_length = input.read_i16::<LE>()? as usize;
            let hints = read_str(input, hints_length)?;
            let win_message_length = input.read_i16::<LE>()? as usize;
            let win_message = read_str(input, win_message_length)?;
            let loss_message_length = input.read_i16::<LE>()? as usize;
            let loss_message = read_str(input, loss_message_length)?;
            let history_length = input.read_i16::<LE>()? as usize;
            let history = read_str(input, history_length)?;
            (hints, win_message, loss_message, history)
        } else {
            (None, None, None, None)
        };

        let scout = if version >= 1.22 {
            let scout_length = input.read_i16::<LE>()? as usize;
            read_str(input, scout_length)?
        } else {
            None
        };

        if version < 1.03 {
            // skip some stuff
        }

        let len = input.read_i16::<LE>()? as usize;
        let pregame_cinematic = read_str(input, len)?;
        let len = input.read_i16::<LE>()? as usize;
        let victory_cinematic = read_str(input, len)?;
        let len = input.read_i16::<LE>()? as usize;
        let loss_cinematic = read_str(input, len)?;

        let mission_bmp = if version >= 1.09 {
            let len = input.read_i16::<LE>()? as usize;
            read_str(input, len)?
        } else {
            None
        };

        let _mission_picture = if version >= 1.10 {
            Bitmap::from(input)?
        } else {
            None
        };

        let mut player_build_lists = vec![None; 16];
        for build_list in player_build_lists.iter_mut() {
            let len = input.read_u16::<LE>()? as usize;
            *build_list = read_str(input, len)?;
        }

        let mut player_city_plans = vec![None; 16];
        for city_plan in player_city_plans.iter_mut() {
            let len = input.read_u16::<LE>()? as usize;
            *city_plan = read_str(input, len)?;
        }

        let mut player_ai_rules = vec![None; 16];
        if version >= 1.08 {
            for ai_rules in player_ai_rules.iter_mut() {
                let len = input.read_u16::<LE>()? as usize;
                *ai_rules = read_str(input, len)?;
            }
        }

        let mut player_files = vec![PlayerFiles::default(); 16];
        for files in player_files.iter_mut() {
            let build_list_length = input.read_i32::<LE>()? as usize;
            let city_plan_length = input.read_i32::<LE>()? as usize;
            let ai_rules_length = if version >= 1.08 {
                input.read_i32::<LE>()? as usize
            } else {
                0
            };

            files.build_list = read_str(input, build_list_length)?;
            files.city_plan = read_str(input, city_plan_length)?;
            files.ai_rules = read_str(input, ai_rules_length)?;
        }

        let mut ai_rules_types = vec![0; 16];
        if version >= 1.20 {
            for rule_type in ai_rules_types.iter_mut() {
                *rule_type = input.read_i8()?;
            }
        }

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        Ok(RGEScen {
            version,
            player_names,
            player_string_table,
            player_base_properties,
            victory_conquest,
            name,
            description_string_table,
            hints_string_table,
            win_message_string_table,
            loss_message_string_table,
            history_string_table,
            scout_string_table,
            description,
            hints,
            win_message,
            loss_message,
            history,
            scout,
            pregame_cinematic,
            victory_cinematic,
            loss_cinematic,
            mission_bmp,
            player_build_lists,
            player_city_plans,
            player_ai_rules,
            player_files,
            ai_rules_types,
        })
    }

    pub fn write_to<W: Write>(&self, mut output: &mut W, version: f32) -> Result<()> {
        output.write_f32::<LE>(version)?;

        if version > 1.13 {
            assert_eq!(self.player_names.len(), 16);
            for name in &self.player_names {
                let mut padded_bytes = Vec::with_capacity(256);
                if let Some(ref name) = name {
                    let name_bytes = name.as_bytes();
                    padded_bytes.write_all(name_bytes)?;
                }
                padded_bytes.extend(vec![0; 256 - padded_bytes.len()]);
                output.write_all(&padded_bytes)?;
            }
        }

        if version > 1.16 {
            assert_eq!(self.player_string_table.len(), 16);
            for id in &self.player_string_table {
                write_opt_string_key(&mut output, id)?;
            }
        }

        if version > 1.13 {
            assert_eq!(self.player_base_properties.len(), 16);
            for props in &self.player_base_properties {
                output.write_i32::<LE>(props.active)?;
                output.write_i32::<LE>(props.player_type)?;
                output.write_i32::<LE>(props.civilization)?;
                output.write_i32::<LE>(props.posture)?;
            }
        }

        if version >= 1.07 {
            output.write_u8(if self.victory_conquest { 1 } else { 0 })?;
        }

        // RGE_Timeline
        output.write_i16::<LE>(0)?;
        output.write_i16::<LE>(0)?;
        output.write_f32::<LE>(-1.0)?;

        write_str(output, &self.name)?;

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

        write_opt_str(output, &self.description)?;
        if version >= 1.11 {
            write_opt_str(output, &self.hints)?;
            write_opt_str(output, &self.win_message)?;
            write_opt_str(output, &self.loss_message)?;
            write_opt_str(output, &self.history)?;
        }
        if version >= 1.22 {
            write_opt_str(output, &self.scout)?;
        }

        write_opt_str(output, &self.pregame_cinematic)?;
        write_opt_str(output, &self.victory_cinematic)?;
        write_opt_str(output, &self.loss_cinematic)?;
        if version >= 1.09 {
            // mission_bmp
            write_opt_str(output, &None)?;
        }

        if version >= 1.10 {
            // mission_picture
            output.write_u32::<LE>(0)?;
            output.write_u32::<LE>(0)?;
            output.write_u32::<LE>(0)?;
            output.write_u16::<LE>(1)?;
        }

        assert_eq!(self.player_build_lists.len(), 16);
        for build_list in &self.player_build_lists {
            write_opt_str(output, build_list)?;
        }

        assert_eq!(self.player_city_plans.len(), 16);
        for city_plan in &self.player_city_plans {
            write_opt_str(output, city_plan)?;
        }

        if version >= 1.08 {
            assert_eq!(self.player_ai_rules.len(), 16);
            for ai_rules in &self.player_ai_rules {
                write_opt_str(output, ai_rules)?;
            }
        }

        assert_eq!(self.player_files.len(), 16);
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
            assert_eq!(self.ai_rules_types.len(), 16);
            for ai_rules_type in &self.ai_rules_types {
                output.write_i8(*ai_rules_type)?;
            }
        }

        output.write_i32::<LE>(-99)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TribeScen {
    /// "Engine" data.
    ///
    /// This distinction doesn't make much sense as a user of this library, but
    /// it exists internally in AoC and affects the storage format (eg.  some
    /// things are duplicate).
    pub(crate) base: RGEScen,
    /// Starting resources for players.
    player_start_resources: Vec<PlayerStartResources>,
    /// Victory settings.
    victory: VictoryInfo,
    /// Whether all victory conditions need to be met for victory to occur.
    victory_all_flag: bool,
    /// Type of victory condition to use in multiplayer games.
    mp_victory_type: i32,
    /// Required score to attain multiplayer victory.
    victory_score: i32,
    /// Time at which the highest-scoring player will win the multiplayer match.
    victory_time: i32,
    /// Initial diplomacy stances between players.
    diplomacy: Vec<Vec<DiplomaticStance>>,
    legacy_victory_info: Vec<Vec<LegacyVictoryInfo>>,
    /// Whether Allied Victory is enabled for each player.
    allied_victory: Vec<i32>,
    teams_locked: bool,
    can_change_teams: bool,
    random_start_locations: bool,
    max_teams: u8,
    /// Number of disabled techs per player.
    num_disabled_techs: Vec<i32>,
    /// Disabled tech IDs per player.
    disabled_techs: Vec<Vec<i32>>,
    /// Number of disabled units per player.
    num_disabled_units: Vec<i32>,
    /// Disabled unit IDs per player.
    disabled_units: Vec<Vec<i32>>,
    /// Number of disabled buildings per player.
    num_disabled_buildings: Vec<i32>,
    /// Disabled building IDs per player.
    disabled_buildings: Vec<Vec<i32>>,
    /// Some unknown scenario option...
    unknown_scenario_option: i32,
    /// Some unknown scenario option...
    unknown_scenario_option_2: i32,
    /// Whether "All Techs" is enabled.
    all_techs: bool,
    /// The starting age per player.
    player_start_ages: Vec<StartingAge>,
    /// The initial camera location.
    view: (i32, i32),
    /// The map type.
    map_type: Option<i32>,
    base_priorities: Vec<i8>,
}

impl TribeScen {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut base = RGEScen::from(input)?;
        let version = base.version;

        let mut player_start_resources = vec![PlayerStartResources::default(); 16];

        // Moved to RGEScen in 1.13
        if version <= 1.13 {
            for name in base.player_names.iter_mut() {
                *name = read_str(input, 256)?;
            }

            for i in 0..16 {
                let properties = &mut base.player_base_properties[i];
                properties.active = input.read_i32::<LE>()?;
                let resources = PlayerStartResources::from(input, version)?;
                properties.player_type = input.read_i32::<LE>()?;
                properties.civilization = input.read_i32::<LE>()?;
                properties.posture = input.read_i32::<LE>()?;
                player_start_resources[i] = resources;
            }
        } else {
            for resources in player_start_resources.iter_mut() {
                *resources = PlayerStartResources::from(input, version)?;
            }
        }

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        let victory = VictoryInfo::from(input)?;
        let victory_all_flag = input.read_i32::<LE>()? != 0;

        let mp_victory_type = if version >= 1.13 {
            input.read_i32::<LE>()?
        } else {
            4
        };
        let victory_score = if version >= 1.13 {
            input.read_i32::<LE>()?
        } else {
            900
        };
        let victory_time = if version >= 1.13 {
            input.read_i32::<LE>()?
        } else {
            9000
        };

        let mut diplomacy = vec![vec![DiplomaticStance::Neutral; 16]; 16];
        for player_diplomacy in diplomacy.iter_mut() {
            for stance in player_diplomacy.iter_mut() {
                *stance = DiplomaticStance::try_from(input.read_i32::<LE>()?)?;
            }
        }

        let mut legacy_victory_info = vec![vec![LegacyVictoryInfo::default(); 12]; 16];
        for list in legacy_victory_info.iter_mut() {
            for victory_info in list.iter_mut() {
                *victory_info = LegacyVictoryInfo::from(input)?;
            }
        }

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        let mut allied_victory = vec![0i32; 16];
        for setting in allied_victory.iter_mut() {
            *setting = input.read_i32::<LE>()?;
        }

        let (teams_locked, can_change_teams, random_start_locations, max_teams) = if version >= 1.24
        {
            (
                input.read_i8()? != 0,
                input.read_i8()? != 0,
                input.read_i8()? != 0,
                input.read_u8()?,
            )
        } else if cmp_float!(version == 1.23) {
            (input.read_i32::<LE>()? != 0, true, true, 4)
        } else {
            (false, true, true, 4)
        };

        let mut num_disabled_techs = vec![0; 16];
        let mut disabled_techs = vec![vec![]; 16];
        let mut num_disabled_units = vec![0; 16];
        let mut disabled_units = vec![vec![]; 16];
        let mut num_disabled_buildings = vec![0; 16];
        let mut disabled_buildings = vec![vec![]; 16];

        if version >= 1.18 {
            for num in num_disabled_techs.iter_mut() {
                *num = input.read_i32::<LE>()?;
            }
            for player_disabled_techs in disabled_techs.iter_mut() {
                for _ in 0..30 {
                    player_disabled_techs.push(input.read_i32::<LE>()?);
                }
            }

            for num in num_disabled_units.iter_mut() {
                *num = input.read_i32::<LE>()?;
            }
            for player_disabled_units in disabled_units.iter_mut() {
                for _ in 0..30 {
                    player_disabled_units.push(input.read_i32::<LE>()?);
                }
            }

            for num in num_disabled_buildings.iter_mut() {
                *num = input.read_i32::<LE>()?;
            }
            let max_disabled_buildings = if version >= 1.25 { 30 } else { 20 };
            for player_disabled_buildings in disabled_buildings.iter_mut() {
                for _ in 0..max_disabled_buildings {
                    player_disabled_buildings.push(input.read_i32::<LE>()?);
                }
            }
        } else if version > 1.03 {
            // Old scenarios only allowed disabling up to 20 techs per player.
            for i in 0..16 {
                let player_disabled_techs = &mut disabled_techs[i];
                for _ in 0..20 {
                    player_disabled_techs.push(input.read_i32::<LE>()?);
                }
                // The number of disabled techs wasn't stored either, so we need to guess it!
                num_disabled_techs[i] = player_disabled_techs
                    .iter()
                    .position(|val| *val <= 0)
                    .map(|index| (index as i32) + 1)
                    .unwrap_or(0);
            }
        } else {
            // <= 1.03 did not support disabling anything
        }

        let unknown_scenario_option = if version > 1.04 {
            input.read_i32::<LE>()?
        } else {
            0
        };
        let (unknown_scenario_option_2, all_techs) = if version >= 1.12 {
            (input.read_i32::<LE>()?, input.read_i32::<LE>()? != 0)
        } else {
            (0, false)
        };

        let mut player_start_ages = vec![StartingAge::Default; 16];
        if version > 1.05 {
            for start_age in player_start_ages.iter_mut() {
                *start_age = StartingAge::try_from(input.read_i32::<LE>()?, version)?;
            }
        }

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        let view = if version >= 1.19 {
            (input.read_i32::<LE>()?, input.read_i32::<LE>()?)
        } else {
            (-1, -1)
        };

        let map_type = if version >= 1.21 {
            Some(input.read_i32::<LE>()?).and_then(|v| if v != -1 { Some(v) } else { None })
        } else {
            None
        };

        let mut base_priorities = vec![0; 16];
        if version >= 1.24 {
            for priority in base_priorities.iter_mut() {
                *priority = input.read_i8()?;
            }
        }

        Ok(TribeScen {
            base,
            player_start_resources,
            victory,
            victory_all_flag,
            mp_victory_type,
            victory_score,
            victory_time,
            diplomacy,
            legacy_victory_info,
            allied_victory,
            teams_locked,
            can_change_teams,
            random_start_locations,
            max_teams,
            num_disabled_techs,
            disabled_techs,
            num_disabled_units,
            disabled_units,
            num_disabled_buildings,
            disabled_buildings,
            unknown_scenario_option,
            unknown_scenario_option_2,
            all_techs,
            player_start_ages,
            view,
            map_type,
            base_priorities,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: f32) -> Result<()> {
        self.base.write_to(output, version)?;

        if version <= 1.13 {
            assert_eq!(self.base.player_names.len(), 16);
            for name in &self.base.player_names {
                let mut padded_bytes = Vec::with_capacity(256);
                if let Some(ref name) = name {
                    let name_bytes = name.as_bytes();
                    padded_bytes.write_all(name_bytes)?;
                }
                padded_bytes.extend(vec![0; 256 - padded_bytes.len()]);
                output.write_all(&padded_bytes)?;
            }

            assert_eq!(self.base.player_base_properties.len(), 16);
            assert_eq!(self.player_start_resources.len(), 16);
            for i in 0..16 {
                let properties = &self.base.player_base_properties[i];
                let resources = &self.player_start_resources[i];
                output.write_i32::<LE>(properties.active)?;
                resources.write_to(output, version)?;
                output.write_i32::<LE>(properties.player_type)?;
                output.write_i32::<LE>(properties.civilization)?;
                output.write_i32::<LE>(properties.posture)?;
            }
        } else {
            assert_eq!(self.player_start_resources.len(), 16);
            for start_resources in &self.player_start_resources {
                start_resources.write_to(output, version)?;
            }
        }

        if version >= 1.02 {
            output.write_i32::<LE>(-99)?;
        }

        self.victory.write_to(output)?;
        output.write_i32::<LE>(if self.victory_all_flag { 1 } else { 0 })?;

        if version >= 1.13 {
            output.write_i32::<LE>(self.mp_victory_type)?;
            output.write_i32::<LE>(self.victory_score)?;
            output.write_i32::<LE>(self.victory_time)?;
        }

        assert_eq!(self.diplomacy.len(), 16);
        for player_diplomacy in &self.diplomacy {
            assert_eq!(player_diplomacy.len(), 16);
            for stance in player_diplomacy {
                output.write_i32::<LE>((*stance).into())?;
            }
        }

        assert_eq!(self.legacy_victory_info.len(), 16);
        for list in &self.legacy_victory_info {
            for entry in list {
                entry.write_to(output)?;
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
        } else if cmp_float!(version == 1.23) {
            output.write_i32::<LE>(if self.teams_locked { 1 } else { 0 })?;
        }

        let max_disabled_buildings = if version >= 1.25 { 30 } else { 20 };
        if version >= 1.18 {
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
            for start_age in &self.player_start_ages {
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
            assert_eq!(self.base_priorities.len(), 16);
            for priority in &self.base_priorities {
                output.write_i8(*priority)?;
            }
        }

        Ok(())
    }

    pub fn version(&self) -> f32 {
        self.base.version
    }

    pub fn description(&self) -> Option<&str> {
        // Convert String to &str: https://stackoverflow.com/a/31234028
        self.base.description.as_ref().map(|s| &**s)
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
    pub(crate) tribe_scen: TribeScen,
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
            data: self.tribe_scen.version(),
            ..VersionBundle::aoc()
        }
    }

    fn load_121<R: Read>(version: SCXVersion, player_version: f32, input: &mut R) -> Result<Self> {
        let header = SCXHeader::from(input, version)?;

        let mut input = DeflateDecoder::new(input);
        let next_object_id = input.read_i32::<LE>()?;

        let tribe_scen = TribeScen::from(&mut input)?;

        let map = Map::from(&mut input)?;

        let num_players = input.read_i32::<LE>()?;
        let mut world_players = vec![];
        for _ in 1..num_players {
            world_players.push(WorldPlayerData::from(&mut input, player_version)?);
        }

        let mut player_objects = vec![];
        for _ in 0..num_players {
            let mut objects = vec![];
            let num_objects = input.read_u32::<LE>()?;
            for _ in 0..num_objects {
                objects.push(ScenarioObject::from(&mut input, version)?);
            }
            player_objects.push(objects);
        }

        let num_scenario_players = input.read_i32::<LE>()?;
        let mut scenario_players = vec![];
        for _ in 1..num_scenario_players {
            scenario_players.push(ScenarioPlayerData::from(&mut input, player_version)?);
        }

        let triggers = if cmp_scx_version(version, *b"1.14") == Ordering::Less {
            None
        } else {
            Some(TriggerSystem::from(&mut input)?)
        };

        let ai_info = if cmp_scx_version(version, *b"1.17") == Ordering::Greater
            && cmp_scx_version(version, *b"2.00") == Ordering::Less
        {
            AIInfo::from(&mut input)?
        } else {
            None
        };

        Ok(SCXFormat {
            version,
            header,
            next_object_id,
            tribe_scen,
            map,
            world_players,
            player_objects,
            scenario_players,
            triggers,
            ai_info,
        })
    }

    pub fn load_scenario<R: Read>(input: &mut R) -> Result<Self> {
        let mut format_version = [0; 4];
        input.read_exact(&mut format_version)?;
        match &format_version {
            b"1.01" => unimplemented!(),
            b"1.02" => unimplemented!(),
            b"1.03" => unimplemented!(),
            b"1.04" => unimplemented!(),
            b"1.05" => unimplemented!(),
            b"1.06" => unimplemented!(),
            b"1.07" => Self::load_121(format_version, 1.07, input),
            b"1.08" => unimplemented!(),
            b"1.09" | b"1.10" | b"1.11" => Self::load_121(format_version, 1.11, input),
            b"1.12" | b"1.13" | b"1.14" | b"1.15" | b"1.16" => {
                Self::load_121(format_version, 1.12, input)
            }
            b"1.17" => Self::load_121(format_version, 1.14, input),
            b"1.18" | b"1.19" => Self::load_121(format_version, 1.13, input),
            b"1.20" | b"1.21" => Self::load_121(format_version, 1.14, input),
            // Definitive Edition
            b"3.13" => Self::load_121(format_version, 1.14, input),
            _ => Err(Error::UnsupportedFormatVersionError(format_version)),
        }
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: &VersionBundle) -> Result<()> {
        let player_version = match &version.format {
            b"1.07" => 1.07,
            b"1.09" | b"1.10" | b"1.11" => 1.11,
            b"1.12" | b"1.13" | b"1.14" | b"1.15" | b"1.16" => 1.12,
            b"1.18" | b"1.19" => 1.13,
            b"1.14" | b"1.20" | b"1.21" => 1.14,
            _ => panic!(
                "writing version {} is not supported",
                String::from_utf8_lossy(&version.format)
            ),
        };

        output.write_all(&version.format)?;
        self.header
            .write_to(output, version.format, version.header)?;

        let mut output = DeflateEncoder::new(output, Compression::default());
        output.write_i32::<LE>(self.next_object_id)?;

        self.tribe_scen.write_to(&mut output, version.data)?;
        self.map.write_to(&mut output)?;

        output.write_i32::<LE>(self.player_objects.len() as i32)?;
        for player in &self.world_players {
            player.write_to(&mut output, player_version)?;
        }

        for objects in &self.player_objects {
            output.write_i32::<LE>(objects.len() as i32)?;
            for object in objects {
                object.write_to(&mut output, version.format)?;
            }
        }

        output.write_i32::<LE>(self.scenario_players.len() as i32 + 1)?;
        for player in &self.scenario_players {
            player.write_to(&mut output, player_version, version.victory)?;
        }

        if cmp_scx_version(version.format, *b"1.13") == Ordering::Greater {
            let def = TriggerSystem::default();
            let triggers = match self.triggers {
                Some(ref tr) => tr,
                None => &def,
            };
            triggers.write_to(&mut output, version.triggers.unwrap_or(1.6))?;
        }

        if cmp_scx_version(version.format, *b"1.17") == Ordering::Greater
            && cmp_scx_version(version.format, *b"2.00") == Ordering::Less
        {
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
        self.tribe_scen.base.player_names[9]
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

fn write_opt_string_key<W: Write>(output: &mut W, opt_key: &Option<StringKey>) -> Result<()> {
    output.write_i32::<LE>(if let Some(key) = opt_key {
        key.try_into()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
    } else {
        -1
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SCXFormat;
    use crate::VersionBundle;
    use std::{fs::File, io::Cursor};

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=42
    #[test]
    fn oldest_aoe1_scn_on_aoeheaven() {
        let mut f = File::open("test/scenarios/ The Destruction of Rome.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    #[test]
    fn aoe1_beta_scn_reserialize() {
        let mut f = File::open("test/scenarios/Dawn of a New Age.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");

        let mut f = Cursor::new(out);
        let format2 = SCXFormat::load_scenario(&mut f).expect("failed to read");

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
        let mut out = vec![];
        format
            .write_to(&mut out, &VersionBundle::aoc())
            .expect("failed to write");

        let mut f = Cursor::new(out);
        let format2 = SCXFormat::load_scenario(&mut f).expect("failed to read");
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
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    #[test]
    fn aoe1_ror_to_aoc() -> Result<(), Box<dyn std::error::Error>> {
        let mut f = File::open("test/scenarios/El advenimiento de los hunos_.scx")?;
        let format = SCXFormat::load_scenario(&mut f)?;

        let mut out = vec![];
        format.write_to(&mut out, &VersionBundle::aoc())?;

        let format = SCXFormat::load_scenario(&mut Cursor::new(out))?;
        assert_eq!(
            format.version(),
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
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    #[test]
    fn aoc_scx() {
        let mut f = File::open("test/scenarios/Age of Heroes b1-3-5.scx").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }

    #[test]
    fn hd_aoe2scenario() {
        let mut f = File::open("test/scenarios/Year_of_the_Pig.aoe2scenario").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");

        let mut f = std::io::Cursor::new(out);
        let format2 = SCXFormat::load_scenario(&mut f).expect("failed to read");

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
        let mut out = vec![];
        format
            .write_to(&mut out, &format.version())
            .expect("failed to write");
    }
}
