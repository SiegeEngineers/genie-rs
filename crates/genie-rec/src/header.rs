use crate::element::ReadableHeaderElement;
use crate::map::Map;
use crate::player::Player;
use crate::reader::RecordingHeaderReader;
use crate::string_table::StringTable;
use crate::{GameVersion, Result};
use byteorder::{ReadBytesExt, LE};
use genie_scx::TribeScen;
use genie_support::ReadSkipExt;
pub use genie_support::SpriteID;
use std::convert::TryInto;
use std::fmt::{self, Debug};
use std::io::Read;

#[derive(Debug, Default, Clone)]
pub struct AICommand {
    pub command_type: i32,
    pub id: u16,
    pub parameters: [i32; 4],
}

impl ReadableHeaderElement for AICommand {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut cmd = Self::default();
        cmd.command_type = input.read_i32::<LE>()?;
        cmd.id = input.read_u16::<LE>()?;
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
        let mut rule = Self::default();
        rule.in_use = input.read_u32::<LE>()? != 0;
        rule.enable = input.read_u32::<LE>()? != 0;
        rule.rule_id = input.read_u16::<LE>()?;
        rule.next_in_group = input.read_u16::<LE>()?;
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
        let mut list = Self::default();
        list.in_use = input.read_u32::<LE>()? != 0;
        list.id = input.read_i32::<LE>()?;
        list.max_rules = input.read_u16::<LE>()?;
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
        let mut table = Self::default();
        table.max_groups = input.read_u16::<LE>()?;
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
        let mut header = Header::default();
        header.game_version = GameVersion::read_from(input)?;
        header.save_version = input.read_f32::<LE>()?;

        // update reader state
        input.set_version(header.game_version, header.save_version);

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
        let mut particle = Self::default();
        particle.start = input.read_u32::<LE>()?;
        particle.facet = input.read_u32::<LE>()?;
        particle.update = input.read_u32::<LE>()?;
        particle.sprite_id = input.read_u16::<LE>()?.into();
        particle.location = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        particle.flags = input.read_u8()?;
        Ok(particle)
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
