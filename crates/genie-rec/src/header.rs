use crate::map::Map;
use crate::player::Player;
use crate::string_table::StringTable;
use crate::Result;
use byteorder::{ReadBytesExt, LE};
pub use genie_support::SpriteID;
use std::convert::TryInto;
use std::fmt::{self, Debug, Display};
use std::io::Read;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GameVersion([u8; 8]);

impl Default for GameVersion {
    fn default() -> Self {
        Self([0; 8])
    }
}

impl Debug for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", std::str::from_utf8(&self.0).unwrap())
    }
}

impl Display for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", std::str::from_utf8(&self.0).unwrap())
    }
}

impl GameVersion {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut game_version = [0; 8];
        input.read_exact(&mut game_version)?;
        Ok(Self(game_version))
    }
}

#[derive(Debug, Default, Clone)]
pub struct AICommand {
    pub command_type: i32,
    pub id: u16,
    pub parameters: [i32; 4],
}

impl AICommand {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut cmd = Self::default();
        cmd.command_type = input.read_i32::<LE>()?;
        cmd.id = input.read_u16::<LE>()?;
        input.read_u16::<LE>()?;
        for param in cmd.parameters.iter_mut() {
            *param = input.read_i32::<LE>()?;
        }
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

impl AIListRule {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut rule = Self::default();
        rule.in_use = input.read_u32::<LE>()? != 0;
        rule.enable = input.read_u32::<LE>()? != 0;
        rule.rule_id = input.read_u16::<LE>()?;
        rule.next_in_group = input.read_u16::<LE>()?;
        let num_facts = input.read_u8()?;
        let num_facts_actions = input.read_u8()?;
        input.read_u16::<LE>()?;
        for i in 0..16 {
            let cmd = AICommand::read_from(&mut input)?;
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

impl AIList {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut list = Self::default();
        list.in_use = input.read_u32::<LE>()? != 0;
        list.id = input.read_i32::<LE>()?;
        list.max_rules = input.read_u16::<LE>()?;
        let num_rules = input.read_u16::<LE>()?;
        input.read_u32::<LE>()?;
        for _ in 0..num_rules {
            list.rules.push(AIListRule::read_from(&mut input)?);
        }
        Ok(list)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIGroupTable {
    max_groups: u16,
    groups: Vec<u16>,
}

impl AIGroupTable {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
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

impl AIFactState {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
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
        for timers in timers.iter_mut() {
            for val in timers.iter_mut() {
                *val = input.read_i32::<LE>()?;
            }
        }
        for goal in shared_goals.iter_mut() {
            *goal = input.read_u32::<LE>()?;
        }
        for signal in signals.iter_mut() {
            *signal = input.read_u32::<LE>()?;
        }
        for trigger in triggers.iter_mut() {
            *trigger = input.read_u32::<LE>()?;
        }
        for taunts in taunts.iter_mut() {
            for taunt in taunts.iter_mut() {
                *taunt = input.read_i8()?;
            }
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
    string_table: StringTable,
    lists: Vec<AIList>,
    groups: Vec<AIGroupTable>,
    fact_state: AIFactState,
}

impl AIScripts {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let string_table = StringTable::read_from(&mut input)?;
        let max_facts = input.read_u16::<LE>()?;
        let max_actions = input.read_u16::<LE>()?;
        let max_lists = input.read_u16::<LE>()?;

        let mut lists = vec![];
        for _ in 0..max_lists {
            lists.push(AIList::read_from(&mut input)?);
        }

        let mut groups = vec![];
        for _ in 0..max_lists {
            groups.push(AIGroupTable::read_from(&mut input)?);
        }

        let fact_state = AIFactState::read_from(&mut input)?;

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
}

impl Header {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut header = Header::default();
        header.game_version = GameVersion::read_from(&mut input)?;
        header.save_version = input.read_f32::<LE>()?;

        let includes_ai = input.read_u32::<LE>()? != 0;
        if includes_ai {
            header.ai_scripts = Some(AIScripts::read_from(&mut input)?);
        }

        let _old_time = input.read_u32::<LE>()?;
        let _world_time = input.read_u32::<LE>()?;
        let _old_world_time = input.read_u32::<LE>()?;
        let _world_time_delta = input.read_u32::<LE>()?;
        let _world_time_delta_seconds = input.read_f32::<LE>()?;
        let _timer = input.read_f32::<LE>()?;
        let game_speed = input.read_f32::<LE>()?;
        let _temp_pause = input.read_i8()?;
        let next_object_id = input.read_u32::<LE>()?;
        let next_reusable_object_id = input.read_i32::<LE>()?;
        let random_seed = input.read_u32::<LE>()?;
        let random_seed2 = input.read_u32::<LE>()?;
        let current_player = input.read_u16::<LE>()?;
        let num_players = input.read_u16::<LE>()?;
        let aegis_enabled = input.read_u8()? != 0;
        let cheats_enabled = input.read_u8()? != 0;
        let game_mode = input.read_u8()?;
        let campaign = input.read_u32::<LE>()?;
        let campaign_player = input.read_u32::<LE>()?;
        let campaign_scenario = input.read_u32::<LE>()?;
        let king_campaign = input.read_u32::<LE>()?;
        let king_campaign_player = input.read_u8()?;
        let king_campaign_scenario = input.read_u8()?;
        let player_turn = input.read_u32::<LE>()?;
        let mut player_time_delta = [0; 9];
        for time_delta in player_time_delta.iter_mut() {
            *time_delta = input.read_u32::<LE>()?;
        }

        header.map = Map::read_from(&mut input)?;

        header.particle_system = ParticleSystem::read_from(&mut input)?;
        let _identifier = dbg!(input.read_u32::<LE>()?);

        let mut players = Vec::with_capacity(num_players.try_into().unwrap());
        players.push(Player::read_from(&mut input, header.save_version, num_players as u8)?);

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

impl Particle {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
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

impl ParticleSystem {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let world_time = input.read_u32::<LE>()?;
        let num_particles = input.read_u32::<LE>()?;
        let mut particles = Vec::with_capacity(num_particles.try_into().unwrap());
        for _ in 0..num_particles {
            particles.push(Particle::read_from(&mut input)?);
        }
        Ok(Self {
            world_time,
            particles,
        })
    }
}
