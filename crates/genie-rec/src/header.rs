use crate::element::ReadableHeaderElement;
use crate::game_options::{Difficulty, GameMode, StartingResources, VictoryType};
use crate::map::Map;
use crate::player::Player;
use crate::reader::RecordingHeaderReader;
use crate::string_table::StringTable;
use crate::GameVariant::DefinitiveEdition;
use crate::{GameVersion, Result};
use byteorder::{ReadBytesExt, LE};
use genie_scx::{AgeIdentifier, TribeScen};
pub use genie_support::SpriteID;
use genie_support::{ReadSkipExt, ReadStringsExt};
use std::convert::TryInto;
use std::fmt::{self, Debug};
use std::io::Read;

const DE_HEADER_SEPARATOR: u32 = u32::from_le_bytes(*b"\xa3_\x02\x00");
const DE_STRING_SEPARATOR: u16 = u16::from_le_bytes(*b"\x60\x0A");
const DE_PLAYER_SEPARATOR: u32 = u32::from_le_bytes(*b"\x00\x00\x00\x00");

#[derive(Debug, Default, Clone)]
pub struct AICommand {
    pub command_type: i32,
    pub id: u16,
    pub parameters: [i32; 4],
}

impl ReadableHeaderElement for AICommand {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut cmd = AICommand {
            command_type: input.read_i32::<LE>()?,
            id: input.read_u16::<LE>()?,
            ..Default::default()
        };

        input.skip(2)?;
        input.read_i32_into::<LE>(&mut cmd.parameters)?;
        Ok(cmd)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIListRule {
    in_use: bool,
    enable: bool,
    rule_id: u16,
    next_in_group: u16,
    facts: Vec<AICommand>,
    actions: Vec<AICommand>,
}

impl ReadableHeaderElement for AIListRule {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut rule = AIListRule {
            in_use: input.read_u32::<LE>()? != 0,
            enable: input.read_u32::<LE>()? != 0,
            rule_id: input.read_u16::<LE>()?,
            next_in_group: input.read_u16::<LE>()?,
            ..Default::default()
        };
        let num_facts = input.read_u8()?;
        let num_facts_actions = input.read_u8()?;
        input.read_u16::<LE>()?;
        for i in 0..16 {
            let cmd = AICommand::read_from(input)?;
            if i < num_facts {
                rule.facts.push(cmd);
            } else if i < num_facts_actions {
                rule.actions.push(cmd);
            }
        }
        Ok(rule)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIList {
    in_use: bool,
    id: i32,
    max_rules: u16,
    rules: Vec<AIListRule>,
}

impl ReadableHeaderElement for AIList {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut list = AIList {
            in_use: input.read_u32::<LE>()? != 0,
            id: input.read_i32::<LE>()?,
            max_rules: input.read_u16::<LE>()?,
            ..Default::default()
        };
        let num_rules = input.read_u16::<LE>()?;
        input.read_u32::<LE>()?;
        for _ in 0..num_rules {
            list.rules.push(AIListRule::read_from(input)?);
        }
        Ok(list)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIGroupTable {
    max_groups: u16,
    groups: Vec<u16>,
}

impl ReadableHeaderElement for AIGroupTable {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut table = AIGroupTable {
            max_groups: input.read_u16::<LE>()?,
            ..Default::default()
        };

        let num_groups = input.read_u16::<LE>()?;
        input.read_u32::<LE>()?;
        for _ in 0..num_groups {
            table.groups.push(input.read_u16::<LE>()?);
        }
        Ok(table)
    }
}

#[derive(Clone)]
pub struct AIFactState {
    pub save_version: f32,
    pub version: f32,
    pub death_match: bool,
    pub regicide: bool,
    pub map_size: u8,
    pub map_type: u8,
    pub starting_resources: u8,
    pub starting_age: u8,
    pub cheats_enabled: bool,
    pub difficulty: u8,
    pub timers: [[i32; 10]; 8],
    pub shared_goals: [u32; 256],
    pub signals: [u32; 256],
    pub triggers: [u32; 256],
    pub taunts: [[i8; 256]; 8],
}

impl Debug for AIFactState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AIFactState")
            .field("save_version", &self.save_version)
            .field("version", &self.version)
            .field("death_match", &self.death_match)
            .field("regicide", &self.regicide)
            .field("map_size", &self.map_size)
            .field("map_type", &self.map_type)
            .field("starting_resources", &self.starting_resources)
            .field("starting_age", &self.starting_age)
            .field("cheats_enabled", &self.cheats_enabled)
            .field("difficulty", &self.difficulty)
            .field("timers", &"...")
            .field("shared_goals", &"...")
            .field("signals", &"...")
            .field("triggers", &"...")
            .field("taunts", &"...")
            .finish()
    }
}

impl ReadableHeaderElement for AIFactState {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let save_version = input.read_f32::<LE>()?;
        let version = input.read_f32::<LE>()?;
        let death_match = input.read_u32::<LE>()? != 0;
        let regicide = input.read_u32::<LE>()? != 0;
        let map_size = input.read_u32::<LE>()?.try_into().unwrap();
        let map_type = input.read_u32::<LE>()?.try_into().unwrap();
        let starting_resources = input.read_u32::<LE>()?.try_into().unwrap();
        let starting_age = input.read_u32::<LE>()?.try_into().unwrap();
        let cheats_enabled = input.read_u32::<LE>()? != 0;
        let difficulty = input.read_u32::<LE>()?.try_into().unwrap();
        let mut timers = [[0; 10]; 8];
        let mut shared_goals = [0; 256];
        let mut signals = [0; 256];
        let mut triggers = [0; 256];
        let mut taunts = [[0; 256]; 8];
        for timer_values in timers.iter_mut() {
            input.read_i32_into::<LE>(&mut timer_values[..])?;
        }
        input.read_u32_into::<LE>(&mut shared_goals)?;
        input.read_u32_into::<LE>(&mut signals)?;
        input.read_u32_into::<LE>(&mut triggers)?;
        for taunts in taunts.iter_mut() {
            input.read_i8_into(&mut taunts[..])?;
        }

        Ok(Self {
            save_version,
            version,
            death_match,
            regicide,
            map_size,
            map_type,
            starting_resources,
            starting_age,
            cheats_enabled,
            difficulty,
            timers,
            shared_goals,
            signals,
            triggers,
            taunts,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AIScripts {
    pub string_table: StringTable,
    pub lists: Vec<AIList>,
    pub groups: Vec<AIGroupTable>,
    pub fact_state: AIFactState,
}

impl ReadableHeaderElement for AIScripts {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let string_table = StringTable::read_from(input)?;
        let _max_facts = input.read_u16::<LE>()?;
        let _max_actions = input.read_u16::<LE>()?;
        let max_lists = input.read_u16::<LE>()?;

        let mut lists = vec![];
        for _ in 0..max_lists {
            lists.push(AIList::read_from(input)?);
        }

        let mut groups = vec![];
        for _ in 0..max_lists {
            groups.push(AIGroupTable::read_from(input)?);
        }

        let fact_state = AIFactState::read_from(input)?;

        Ok(AIScripts {
            string_table,
            lists,
            groups,
            fact_state,
        })
    }
}

#[derive(Debug, Default)]
pub struct Header {
    game_version: GameVersion,
    save_version: f32,
    de_extension_header: Option<DeExtensionHeader>,
    ai_scripts: Option<AIScripts>,
    map: Map,
    particle_system: ParticleSystem,
    players: Vec<Player>,
    scenario: TribeScen,
}

impl Header {
    pub fn players(&self) -> impl Iterator<Item = &Player> {
        self.players.iter()
    }
}

impl ReadableHeaderElement for Header {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut header = Header {
            game_version: GameVersion::read_from(input)?,
            save_version: input.read_f32::<LE>()?,
            ..Default::default()
        };

        // update reader state
        input.set_version(header.game_version, header.save_version);

        if input.variant() >= DefinitiveEdition {
            header.de_extension_header = Some(DeExtensionHeader::read_from(input)?)
        }

        let includes_ai = input.read_u32::<LE>()? != 0;
        if includes_ai {
            header.ai_scripts = Some(AIScripts::read_from(input)?);
        }

        let _old_time = input.read_u32::<LE>()?;
        let _world_time = input.read_u32::<LE>()?;
        let _old_world_time = input.read_u32::<LE>()?;
        let _world_time_delta = input.read_u32::<LE>()?;
        let _world_time_delta_seconds = input.read_f32::<LE>()?;
        let _timer = input.read_f32::<LE>()?;
        let _game_speed = input.read_f32::<LE>()?;
        let _temp_pause = input.read_i8()?;
        let _next_object_id = input.read_u32::<LE>()?;
        let _next_reusable_object_id = input.read_i32::<LE>()?;
        let _random_seed = input.read_u32::<LE>()?;
        let _random_seed2 = input.read_u32::<LE>()?;
        let _current_player = input.read_u16::<LE>()?;
        let num_players = input.read_u16::<LE>()?;
        input.set_num_players(num_players);
        if input.version() >= 11.76 {
            let _aegis_enabled = input.read_u8()? != 0;
            let _cheats_enabled = input.read_u8()? != 0;
        }
        let _game_mode = input.read_u8()?;
        let _campaign = input.read_u32::<LE>()?;
        let _campaign_player = input.read_u32::<LE>()?;
        let _campaign_scenario = input.read_u32::<LE>()?;
        if input.version() >= 10.13 {
            let _king_campaign = input.read_u32::<LE>()?;
            let _king_campaign_player = input.read_u8()?;
            let _king_campaign_scenario = input.read_u8()?;
        }
        let _player_turn = input.read_u32::<LE>()?;
        let mut player_time_delta = [0; 9];
        input.read_u32_into::<LE>(&mut player_time_delta[..])?;

        if header.save_version >= 12.97 {
            // ???
            input.skip(8)?;
        }

        header.map = Map::read_from(input)?;

        // TODO is there another num_players here for restored games?

        header.particle_system = ParticleSystem::read_from(input)?;

        if header.save_version >= 11.07 {
            let _identifier = input.read_u32::<LE>()?;
        }

        header.players.reserve(num_players.try_into().unwrap());
        for _ in 0..num_players {
            header.players.push(Player::read_from(input)?);
        }
        for player in &mut header.players {
            player.read_info(input)?;
        }

        header.scenario = TribeScen::read_from(&mut *input)?;

        if input.variant() >= DefinitiveEdition {
            input.skip(133)?;
        }

        let _difficulty = if header.save_version >= 7.16 {
            Some(input.read_u32::<LE>()?)
        } else {
            None
        };
        let _lock_teams = if header.save_version >= 10.23 {
            input.read_u32::<LE>()? != 0
        } else {
            false
        };

        if header.save_version >= 11.32 {
            for _ in 0..9 {
                let _player_id = input.read_u32::<LE>()?;
                let _player_humanity = input.read_u32::<LE>()?;
                let name_length = input.read_u32::<LE>()?;
                let mut name = vec![0; name_length as usize];
                input.read_exact(&mut name)?;
            }
        }

        if header.save_version >= 11.35 {
            for _ in 0..9 {
                let _resigned = input.read_u32::<LE>()?;
            }
        }

        if header.save_version >= 11.36 {
            let _num_players = input.read_u32::<LE>()?;
        }

        if header.save_version >= 11.38 {
            let _sent_commanded_count = input.read_u32::<LE>()?;
            if header.save_version >= 11.39 {
                let _sent_commanded_valid = input.read_u32::<LE>()?;
            }
            let mut sent_commanded_units = [0u32; 40];
            input.read_u32_into::<LE>(&mut sent_commanded_units)?;
            for _ in 0..9 {
                let _num_selected = input.read_u8()?;
                let mut selection = [0u32; 40];
                input.read_u32_into::<LE>(&mut selection)?;
            }
        }

        let _num_paths = input.read_u32::<LE>()?;
        // TODO: Read paths
        // TODO: Read unit groups

        Ok(header)
    }
}

#[derive(Debug, Default, Clone)]
pub struct DeExtensionHeader {
    pub build: Option<f32>,     // save_version >= 25.22
    pub timestamp: Option<f32>, // save_version >= 26.16
    pub version: f32,
    pub interval_version: u32,
    pub game_options_version: u32,
    pub dlc_count: u32,
    pub dlc_ids: Vec<u32>,
    pub dataset_ref: u32,
    pub difficulty: Difficulty, // unsure, always "4"
    pub selected_map_id: u32,
    pub resolved_map_id: u32,
    pub reveal_map: u32,
    pub victory_type_id: u32,
    pub victory_type: VictoryType,
    pub starting_resources_id: i32,
    pub starting_resources: StartingResources,
    pub starting_age_id: i32,
    pub starting_age: AgeIdentifier,
    pub ending_age_id: i32,
    pub ending_age: AgeIdentifier,
    pub game_mode: GameMode,
    // DE_HEADER_SEPARATOR,
    // DE_HEADER_SEPARATOR,
    pub speed: f32,
    pub treaty_length: u32,
    pub population_limit: u32,
    pub num_players: u32,
    pub unused_player_color: u32,
    pub victory_amount: u32,
    // DE_HEADER_SEPARATOR,
    pub trade_enabled: bool,
    pub team_bonus_disabled: bool,
    pub random_positions: bool,
    pub all_techs: bool,
    pub num_starting_units: u8,
    pub lock_teams: bool,
    pub lock_speed: bool,
    pub multiplayer: bool,
    pub cheats_enabled: bool,
    pub record_game: bool,
    pub animals_enabled: bool,
    pub predators_enabled: bool,
    pub turbo_enabled: bool,
    pub shared_exploration: bool,
    pub team_positions: bool,
    pub sub_game_mode: Option<u32>,      // save_version >= 13.34
    pub battle_royale_time: Option<u32>, // save_version >= 13.34
    pub handicap: Option<bool>,          // save_version >= 25.06
    // DE_HEADER_SEPARATOR,
    pub players: [DePlayer; 8],
    // 9 bytes
    pub fog_of_war: bool,
    pub cheat_notifications: bool,
    pub colored_chat: bool,
    // DE_HEADER_SEPARATOR
    pub ranked: bool,
    pub allow_spectators: bool,
    pub lobby_visibility: u32,
    pub hidden_civs: bool,
    pub matchmaking: bool,
    pub spectator_delay: u32,
    pub scenario_civ: Option<u8>, // save_version >= 13.13
    pub rms_crc: Option<[u8; 4]>, // save_version >= 13.13
    // Skipped for now, check https://github.com/happyleavesaoc/aoc-mgz/blob/44cd0a6d8ea19524c82893f11be928b468c46bea/mgz/header/de.py#L111
    // 8 bytes; save_version >= 25.02
    pub num_ai_files: i64,
    // TODO: ai_files, skipped
    pub guid: u128,
    pub lobby_name: String,
    // 8 bytes; save_version >= 25.22
    pub modded_dataset: String,
    // 19 bytes
    // 5 bytes; save_version >= 13.13
    // 9 bytes; save_version >= 13.17
    // 1 bytes; save_version >= 20.06
    // 8 bytes; save_version >= 20.16
    // 21 bytes; save_version >= 25.06
    // 4 bytes; save_version >= 25.22
    // 8 bytes; save_version >= 26.16
    // DeString
    // 5 bytes
    // 1 byte; save_version >= 13.13
    // Struct
    // 2 byte; save_version >= 13.17
}

impl ReadableHeaderElement for DeExtensionHeader {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut header = Self::default();
        if input.version() >= 25.22 {
            header.build = Some(input.read_f32::<LE>()?)
        } else {
            header.build = None
        };

        if input.version() >= 26.16 {
            header.timestamp = Some(input.read_f32::<LE>()?)
        } else {
            header.timestamp = None
        };

        header.version = input.read_f32::<LE>()?;
        header.interval_version = input.read_u32::<LE>()?;
        header.game_options_version = input.read_u32::<LE>()?;
        header.dlc_count = input.read_u32::<LE>()?;

        for _ in 0..header.dlc_count {
            header.dlc_ids.push(input.read_u32::<LE>()?)
        }

        header.dataset_ref = input.read_u32::<LE>()?;
        header.difficulty = input.read_u32::<LE>()?.into();
        header.selected_map_id = input.read_u32::<LE>()?;
        header.resolved_map_id = input.read_u32::<LE>()?;
        header.reveal_map = input.read_u32::<LE>()?;
        header.victory_type_id = input.read_u32::<LE>()?;
        header.victory_type = header.victory_type_id.into();
        header.starting_resources_id = input.read_i32::<LE>()?;
        header.starting_resources = header.starting_resources_id.into();
        header.starting_age_id = input.read_i32::<LE>()?;
        header.starting_age = AgeIdentifier::try_from(header.starting_age_id, input.version())
            .expect("Converting starting age identifier failed.");
        header.ending_age_id = input.read_i32::<LE>()?;
        header.ending_age = AgeIdentifier::try_from(header.ending_age_id, input.version())
            .expect("Converting ending age identifier failed.");
        header.game_mode = input.read_u32::<LE>()?.into();
        assert_eq!(input.read_u32::<LE>()?, DE_HEADER_SEPARATOR);
        assert_eq!(input.read_u32::<LE>()?, DE_HEADER_SEPARATOR);
        header.speed = input.read_f32::<LE>()?;
        header.treaty_length = input.read_u32::<LE>()?;
        header.population_limit = input.read_u32::<LE>()?;
        header.num_players = input.read_u32::<LE>()?;
        header.unused_player_color = input.read_u32::<LE>()?;
        header.victory_amount = input.read_u32::<LE>()?;
        assert_eq!(input.read_u32::<LE>()?, DE_HEADER_SEPARATOR);
        header.trade_enabled = input.read_u8()? == 1;
        header.team_bonus_disabled = input.read_u8()? == 1;
        header.random_positions = input.read_u8()? == 1;
        header.all_techs = input.read_u8()? == 1;
        header.num_starting_units = input.read_u8()?;
        header.lock_teams = input.read_u8()? == 1;
        header.lock_speed = input.read_u8()? == 1;
        header.multiplayer = input.read_u8()? == 1;
        header.cheats_enabled = input.read_u8()? == 1;
        header.record_game = input.read_u8()? == 1;
        header.animals_enabled = input.read_u8()? == 1;
        header.predators_enabled = input.read_u8()? == 1;
        header.turbo_enabled = input.read_u8()? == 1;
        header.shared_exploration = input.read_u8()? == 1;
        header.team_positions = input.read_u8()? == 1;
        if input.version() >= 13.34 {
            header.sub_game_mode = Some(input.read_u32::<LE>()?)
        } else {
            header.sub_game_mode = None
        };

        if input.version() >= 13.34 {
            header.battle_royale_time = Some(input.read_u32::<LE>()?)
        } else {
            header.battle_royale_time = None
        };

        if input.version() >= 25.06 {
            header.handicap = Some(input.read_u8()? == 1)
        } else {
            header.handicap = None
        };

        assert_eq!(input.read_u32::<LE>()?, DE_HEADER_SEPARATOR);

        // TODO DEBUG
        dbg!(&header);

        for i in 0..header.num_players as usize {
            header.players[i].apply_from(input)?;
        }

        // Skip 9 unknown bytes
        input.skip(9)?;

        header.fog_of_war = input.read_u8()? == 1;
        header.cheat_notifications = input.read_u8()? == 1;
        header.colored_chat = input.read_u8()? == 1;

        assert_eq!(input.read_u32::<LE>()?, DE_HEADER_SEPARATOR);

        // input.skip(12)?;

        header.ranked = input.read_u8()? == 1;
        header.allow_spectators = input.read_u8()? == 1;
        header.lobby_visibility = input.read_u32::<LE>()?;
        header.hidden_civs = input.read_u8()? == 1;
        header.matchmaking = input.read_u8()? == 1;
        header.spectator_delay = input.read_u32::<LE>()?;

        if input.version() >= 13.13 {
            header.scenario_civ = Some(input.read_u8()?)
        } else {
            header.scenario_civ = None
        };

        if input.version() >= 13.13 {
            let mut temp = [0u8; 4];
            #[allow(clippy::needless_range_loop)]
            for i in 0..temp.len() {
                temp[i] = input.read_u8()?
            }
            header.rms_crc = Some(temp)
        } else {
            header.rms_crc = None
        };

        // TODO: read strings
        for _ in 0..23 {
            let _string = input.read_hd_style_str()?;
            while [3, 21, 23, 42, 44, 45].contains(&input.read_u32::<LE>()?) {}
        }

        // TODO "strategic numbers" ???
        input.skip(59 * 4)?;

        // num ai files
        header.num_ai_files = input.read_i64::<LE>()?;

        for _ in 0..header.num_ai_files {
            input.skip(4)?;
            input.read_hd_style_str()?;
            input.skip(4)?;
        }

        if input.version() >= 25.02 {
            input.skip(8)?;
        }

        header.guid = input.read_u128::<LE>()?;

        header.lobby_name = input.read_hd_style_str()?.unwrap_or_default();

        if input.version() >= 25.22 {
            input.skip(8)?;
        }

        header.modded_dataset = input.read_hd_style_str()?.unwrap_or_default();

        input.skip(19)?;

        if input.version() >= 13.13 {
            input.skip(5)?;
        }

        if input.version() >= 13.17 {
            input.skip(9)?;
        }

        if input.version() >= 20.06 {
            input.skip(1)?;
        }

        if input.version() >= 20.16 {
            input.skip(8)?;
        }

        if input.version() >= 25.06 {
            input.skip(21)?;
        }

        if input.version() >= 25.22 {
            input.skip(4)?;
        }

        if input.version() >= 26.16 {
            input.skip(8)?;
        }

        input.read_hd_style_str()?;

        input.skip(5)?;

        if input.version() >= 13.13 {
            input.skip(1)?;
        }

        if input.version() < 13.17 {
            input.read_hd_style_str()?;
            input.skip(4)?;
            input.skip(4)?; // usually 0
        }

        if input.version() >= 13.17 {
            input.skip(2)?;
        }

        Ok(header)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PlayerType {
    Absent = 0,
    Closed = 1,
    Human = 2,
    Eliminated = 3,
    Computer = 4,
    Cyborg = 5,
    Spectator = 6,
    Unknown = 999,
}

impl Default for PlayerType {
    fn default() -> Self {
        PlayerType::Human
    }
}

impl From<u32> for PlayerType {
    #[inline]
    fn from(condition: u32) -> PlayerType {
        match condition {
            0 => PlayerType::Absent,
            1 => PlayerType::Closed,
            2 => PlayerType::Human,
            3 => PlayerType::Eliminated,
            4 => PlayerType::Computer,
            5 => PlayerType::Cyborg,
            6 => PlayerType::Spectator,
            7..=u32::MAX => PlayerType::Unknown,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DePlayer {
    dlc_id: u32,
    color_id: i32,
    selected_color: i8,
    selected_team_id: u8,
    resolved_team_id: u8,
    dat_crc: u64,
    mp_game_version: u8,
    civ_id: u8,
    ai_type: String,
    ai_civ_name_index: u8,
    ai_name: String,
    name: String,
    player_type: PlayerType,
    profile_id: u32,
    // DE_PLAYER_SEPARATOR,
    player_number: i32,
    hd_rm_elo: Option<u32>, // save_version < 25.22
    hd_dm_elo: Option<u32>, // save_version < 25.22
    prefer_random: bool,
    custom_ai: bool,
    handicap: Option<u8>, // save_version < 25.06
}

impl DePlayer {
    pub fn apply_from<R: Read>(&mut self, input: &mut RecordingHeaderReader<R>) -> Result<()> {
        self.dlc_id = input.read_u32::<LE>()?;
        self.color_id = input.read_i32::<LE>()?;
        self.selected_color = input.read_i8()?;
        self.selected_team_id = input.read_u8()?;
        self.resolved_team_id = input.read_u8()?;
        self.dat_crc = input.read_u64::<LE>()?;
        self.mp_game_version = input.read_u8()?;
        self.civ_id = input.read_u8()?;

        // TODO: Needed?
        // input.skip(3)?;

        self.ai_type = input.read_hd_style_str()?.unwrap_or_default();
        self.ai_civ_name_index = input.read_u8()?;
        self.ai_name = input.read_hd_style_str()?.unwrap_or_default();
        self.name = input.read_hd_style_str()?.unwrap_or_default();
        self.player_type = input.read_u32::<LE>()?.into();
        self.profile_id = input.read_u32::<LE>()?;

        // DE_PLAYER_SEPARATOR
        assert_eq!(input.read_u32::<LE>()?, 0);

        self.player_number = input.read_i32::<LE>()?;

        if input.version() < 25.22 {
            self.hd_rm_elo = Some(input.read_u32::<LE>()?)
        } else {
            self.hd_rm_elo = None
        };

        if input.version() < 25.22 {
            self.hd_dm_elo = Some(input.read_u32::<LE>()?)
        } else {
            self.hd_dm_elo = None
        };

        self.prefer_random = input.read_u8()? == 1;
        self.custom_ai = input.read_u8()? == 1;

        if input.version() < 25.06 {
            self.handicap = Some(input.read_u8()?)
        } else {
            self.handicap = None
        };

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
struct Particle {
    pub start: u32,
    pub facet: u32,
    pub update: u32,
    pub sprite_id: SpriteID,
    pub location: (f32, f32, f32),
    pub flags: u8,
}

impl ReadableHeaderElement for Particle {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        Ok(Particle {
            start: input.read_u32::<LE>()?,
            facet: input.read_u32::<LE>()?,
            update: input.read_u32::<LE>()?,
            sprite_id: input.read_u16::<LE>()?.into(),
            location: (
                input.read_f32::<LE>()?,
                input.read_f32::<LE>()?,
                input.read_f32::<LE>()?,
            ),
            flags: input.read_u8()?,
        })
    }
}

#[derive(Debug, Default, Clone)]
struct ParticleSystem {
    pub world_time: u32,
    pub particles: Vec<Particle>,
}

impl ReadableHeaderElement for ParticleSystem {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let world_time = input.read_u32::<LE>()?;
        let num_particles = input.read_u32::<LE>()?;
        let mut particles = Vec::with_capacity(num_particles.try_into().unwrap());
        for _ in 0..num_particles {
            particles.push(Particle::read_from(input)?);
        }
        Ok(Self {
            world_time,
            particles,
        })
    }
}
