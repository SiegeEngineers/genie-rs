//! Read and write player AI state.

use crate::unit::Waypoint;
use crate::{ObjectID, PlayerID, Result};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{read_opt_u16, read_opt_u32, ReadSkipExt, UnitTypeID};
use std::convert::TryInto;
use std::io::{Read, Write};

/// The main AI module.
#[derive(Debug, Default, Clone)]
pub struct MainAI {
    pub objects: Vec<ObjectID>,
}

impl MainAI {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let num_objects = input.read_u32::<LE>()?;
        let mut objects = vec![];
        for _ in 0..num_objects {
            objects.push(input.read_u32::<LE>()?.into());
        }
        Ok(Self { objects })
    }
}

#[derive(Debug, Default, Clone)]
pub struct BuildItem {
    pub name: Option<String>,
    pub type_id: u32,
    pub game_id: u32,
    pub size: (f32, f32, f32),
    pub skip: u32,
    pub build_category: u32,
    pub in_progress: u32,
    pub built: u32,
    pub build_attempts: u32,
    pub build_from: u32,
    pub terrain_set: u32,
    pub terrain_adjacency: (u32, u32),
    pub place_on_elevation: u32,
    pub is_forward: bool,
}

impl BuildItem {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut item = Self::default();
        item.name = {
            let len = input.read_u32::<LE>()?;
            genie_support::read_str(&mut input, len.try_into().unwrap())?
        };
        item.type_id = input.read_u32::<LE>()?;
        let _a2 = input.read_u32::<LE>()?;
        item.game_id = input.read_u32::<LE>()?;
        let _v21 = input.read_u32::<LE>()?;
        let _v23 = input.read_u32::<LE>()?;
        let _v25 = input.read_u32::<LE>()?;
        item.size = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        item.skip = input.read_u32::<LE>()?;
        item.build_category = input.read_u32::<LE>()?;
        item.in_progress = input.read_u32::<LE>()?;
        item.built = input.read_u32::<LE>()?;
        item.build_attempts = input.read_u32::<LE>()?;
        item.build_from = input.read_u32::<LE>()?;
        item.terrain_set = input.read_u32::<LE>()?;
        item.terrain_adjacency = (input.read_u32::<LE>()?, input.read_u32::<LE>()?);
        item.place_on_elevation = input.read_u32::<LE>()?;
        let _v27 = input.read_u32::<LE>()?;
        let _v12 = input.read_u32::<LE>()?;
        let _v29 = input.read_u32::<LE>()?;
        let _v31 = input.read_u8()?;
        item.is_forward = if version > 10.87 {
            input.read_u32::<LE>()? != 0
        } else {
            false
        };
        Ok(item)
    }
}

#[derive(Debug, Default, Clone)]
pub struct BuildAI {
    pub build_list_name: Option<String>,
    pub last_build_item_requested: Option<String>,
    pub current_build_item_requested: Option<String>,
    pub next_build_item_requested: Option<String>,
    pub build_queue: Vec<BuildItem>,
    pub queued_object_count: Vec<u32>,
    pub queued_building_count: u32,
    pub queued_unit_count: u32,
}

impl BuildAI {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut ai = Self::default();
        let build_list_len = input.read_u32::<LE>()?;
        ai.build_list_name = {
            let len = input.read_u32::<LE>()?;
            genie_support::read_str(&mut input, len.try_into().unwrap())?
        };
        ai.last_build_item_requested = {
            let len = input.read_u32::<LE>()?;
            genie_support::read_str(&mut input, len.try_into().unwrap())?
        };
        ai.current_build_item_requested = {
            let len = input.read_u32::<LE>()?;
            genie_support::read_str(&mut input, len.try_into().unwrap())?
        };
        ai.next_build_item_requested = {
            let len = input.read_u32::<LE>()?;
            genie_support::read_str(&mut input, len.try_into().unwrap())?
        };
        let _items_into_build_queue = if version > 11.02 {
            input.read_u32::<LE>()?
        } else {
            build_list_len
        };

        let num_build_items = input.read_u32::<LE>()?;
        for _ in 0..num_build_items {
            ai.build_queue
                .push(BuildItem::read_from(&mut input, version)?);
        }

        for _ in 0..600 {
            ai.queued_object_count.push(input.read_u32::<LE>()?);
        }
        ai.queued_building_count = input.read_u32::<LE>()?;
        ai.queued_unit_count = input.read_u32::<LE>()?;
        Ok(ai)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ConstructionItem {
    pub name: Option<String>,
    pub type_id: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub xs: f32,
    pub ys: f32,
    pub zs: f32,
    pub skip: u32,
    pub in_progress: u32,
    pub built: u32,
}

impl ConstructionItem {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut item = Self::default();
        item.name = {
            let len = input.read_u32::<LE>()?;
            genie_support::read_str(&mut input, len.try_into().unwrap())?
        };
        item.type_id = input.read_u32::<LE>()?;
        let _a2 = input.read_u32::<LE>()?;
        let _v27 = input.read_u32::<LE>()?;
        item.x = input.read_f32::<LE>()?;
        item.y = input.read_f32::<LE>()?;
        item.z = input.read_f32::<LE>()?;
        item.xs = input.read_f32::<LE>()?;
        item.ys = input.read_f32::<LE>()?;
        item.zs = input.read_f32::<LE>()?;
        item.skip = input.read_u32::<LE>()?;
        item.in_progress = input.read_u32::<LE>()?;
        item.built = input.read_u32::<LE>()?;
        let _v23 = input.read_u32::<LE>()?;
        Ok(item)
    }
}
#[derive(Debug, Default, Clone)]
pub struct ConstructionAI {
    pub plan_name: Option<String>,
    pub reference_point: (f32, f32, f32),
    pub map_size: (u32, u32),
    pub construction_lots: Vec<ConstructionItem>,
    pub random_construction_lots: Vec<ConstructionItem>,
}

impl ConstructionAI {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut ai = Self::default();
        let num_lots = input.read_u32::<LE>()?;
        ai.plan_name = {
            let len = input.read_u32::<LE>()?;
            genie_support::read_str(&mut input, len.try_into().unwrap())?
        };
        ai.reference_point = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        ai.map_size = (input.read_u32::<LE>()?, input.read_u32::<LE>()?);
        for _ in 0..num_lots {
            ai.construction_lots
                .push(ConstructionItem::read_from(&mut input, version)?);
        }
        let num_lots = input.read_u32::<LE>()?;
        for _ in 0..num_lots {
            ai.random_construction_lots
                .push(ConstructionItem::read_from(&mut input, version)?);
        }
        Ok(ai)
    }
}

#[derive(Debug, Default, Clone)]
pub struct DiplomacyAI {
    /// How much this AI dislikes each player, 0-100.
    pub dislike: [u32; 10],
    /// How much this AI likes each player, 0-100.
    pub like: [u32; 10],
    pub changeable: [u8; 10],
}

impl DiplomacyAI {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut ai = Self::default();
        for i in 0..10 {
            ai.dislike[i] = input.read_u32::<LE>()?;
            ai.like[i] = input.read_u32::<LE>()?;
            ai.changeable[i] = input.read_u8()?;
        }
        Ok(ai)
    }
}

#[derive(Debug, Default, Clone)]
pub struct EmotionalAI {
    pub state: [u32; 6],
}

impl EmotionalAI {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut ai = Self::default();
        input.read_u32_into::<LE>(&mut ai.state)?;
        Ok(ai)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ImportantObjectMemory {
    pub id: Option<ObjectID>,
    pub unit_type_id: Option<UnitTypeID>,
    pub unit_class: Option<u16>,
    pub location: (u8, u8, u8),
    pub owner: PlayerID,
    pub hit_points: u16,
    pub attack_attempts: u32,
    pub kills: u8,
    pub damage_capability: f32,
    pub rate_of_fire: f32,
    pub range: f32,
    pub time_seen: Option<u32>,
    pub is_garrisoned: u32,
}

impl ImportantObjectMemory {
    pub fn read_from(mut input: impl Read, _version: f32) -> Result<Self> {
        let mut object = Self::default();
        object.id = read_opt_u32(&mut input)?;
        object.unit_type_id = read_opt_u16(&mut input)?;
        object.unit_class = read_opt_u16(&mut input)?;
        object.location = (input.read_u8()?, input.read_u8()?, input.read_u8()?);
        object.owner = input.read_u8()?.into();
        object.hit_points = input.read_u16::<LE>()?;
        object.attack_attempts = input.read_u32::<LE>()?;
        object.kills = input.read_u8()?;
        object.damage_capability = input.read_f32::<LE>()?;
        object.rate_of_fire = input.read_f32::<LE>()?;
        object.range = input.read_f32::<LE>()?;
        object.time_seen = read_opt_u32(&mut input)?;
        object.is_garrisoned = input.read_u32::<LE>()?;
        Ok(object)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BuildingLot {
    pub unit_type_id: Option<UnitTypeID>,
    pub status: u8,
    pub location: (u8, u8),
}

impl BuildingLot {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let unit_type_id = read_opt_u32(&mut input)?;
        let status = input.read_u8()?;
        let x = input.read_u8()?;
        let y = input.read_u8()?;
        input.skip(1)?;
        Ok(Self {
            unit_type_id,
            status,
            location: (x, y),
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct WallLine {
    pub line_type: u32,
    pub wall_type: Option<u32>,
    pub gate_count: u32,
    pub segment_count: u32,
    pub invisible_segment_count: u32,
    pub unfinished_segment_count: u32,
    pub line_start: (u32, u32),
    pub line_end: (u32, u32),
}

impl WallLine {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut line = Self::default();
        if version >= 10.78 {
            line.line_type = input.read_u32::<LE>()?;
        }
        line.wall_type = read_opt_u32(&mut input)?;
        if version >= 10.78 {
            line.gate_count = input.read_u32::<LE>()?;
        }
        line.segment_count = input.read_u32::<LE>()?;
        if version >= 11.34 {
            line.invisible_segment_count = input.read_u32::<LE>()?;
        }
        if version >= 11.29 {
            line.unfinished_segment_count = input.read_u32::<LE>()?;
        }
        line.line_start = (input.read_u32::<LE>()?, input.read_u32::<LE>()?);
        line.line_end = (input.read_u32::<LE>()?, input.read_u32::<LE>()?);
        Ok(line)
    }
}

#[derive(Debug, Default, Clone)]
pub struct PerimeterWall {
    pub enabled: bool,
    pub lines: Vec<WallLine>,
    pub gate_count: u32,
    pub gate_fitting_line_count: u32,
    pub percentage_complete: u32,
    pub segment_count: u32,
    pub invisible_segment_count: u32,
    pub unfinished_segment_count: u32,
    pub next_line_to_refresh: u32,
    pub next_segment_to_refresh: u32,
}

impl PerimeterWall {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut wall = Self::default();
        wall.enabled = if version >= 11.22 {
            input.read_u32::<LE>()? != 0
        } else {
            true
        };
        let num_lines = input.read_u32::<LE>()?;
        if version >= 11.20 {
            wall.gate_count = input.read_u32::<LE>()?;
            wall.percentage_complete = input.read_u32::<LE>()?;
            wall.segment_count = input.read_u32::<LE>()?;
            if version >= 11.34 {
                wall.invisible_segment_count = input.read_u32::<LE>()?;
            }
            wall.unfinished_segment_count = input.read_u32::<LE>()?;
        }
        if version >= 11.29 {
            wall.next_line_to_refresh = input.read_u32::<LE>()?;
            wall.next_segment_to_refresh = input.read_u32::<LE>()?;
        }

        wall.lines = {
            let mut list = vec![];
            for _ in 0..num_lines {
                list.push(WallLine::read_from(&mut input, version)?);
            }
            list
        };

        Ok(wall)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AttackMemory {
    pub id: Option<i32>,
    pub typ: u8,
    pub min_x: u8,
    pub min_y: u8,
    pub max_x: u8,
    pub max_y: u8,
    pub attacking_owner: Option<u8>,
    pub target_owner: Option<u8>,
    pub kills: u16,
    pub success: bool,
    pub timestamp: Option<u32>,
    pub play: Option<i32>,
}

impl AttackMemory {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut mem = Self::default();
        mem.id = read_opt_u32(&mut input)?;
        mem.typ = input.read_u8()?;
        mem.min_x = input.read_u8()?;
        mem.min_y = input.read_u8()?;
        mem.max_x = input.read_u8()?;
        mem.max_y = input.read_u8()?;
        mem.attacking_owner = match input.read_i8()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        mem.target_owner = match input.read_i8()? {
            -1 => None,
            id => Some(id.try_into().unwrap()),
        };
        input.skip(1)?;
        mem.kills = input.read_u16::<LE>()?;
        mem.success = input.read_u8()? != 0;
        input.skip(1)?;
        mem.timestamp = read_opt_u32(&mut input)?;
        mem.play = read_opt_u32(&mut input)?;
        Ok(mem)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ResourceMemory {
    pub id: ObjectID,
    pub location: (u8, u8),
    pub gather_attempts: u8,
    pub gather: u32,
    pub valid: bool,
    pub gone: bool,
    pub drop_distance: u8,
    pub resource_type: u8,
    pub dropsite_id: ObjectID,
    pub attacked_time: Option<u32>,
}

impl ResourceMemory {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut mem = Self::default();
        mem.id = input.read_u32::<LE>()?.into();
        mem.location = (input.read_u8()?, input.read_u8()?);
        mem.gather_attempts = input.read_u8()?;
        mem.gather = input.read_u32::<LE>()?;
        mem.valid = input.read_u8()? != 0;
        mem.gone = input.read_u8()? != 0;
        mem.drop_distance = input.read_u8()?;
        mem.resource_type = input.read_u8()?;
        mem.dropsite_id = input.read_u32::<LE>()?.into();
        if version >= 10.91 {
            mem.attacked_time = read_opt_u32(&mut input)?;
        }
        Ok(mem)
    }
}

#[derive(Debug, Default, Clone)]
pub struct InfluenceMap {
    pub width: u32,
    pub height: u32,
    pub reference_point: (u32, u32),
    pub unchangeable_limit: u8,
    pub values: Vec<i8>,
}

impl InfluenceMap {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut map = Self::default();
        map.width = input.read_u32::<LE>()?;
        map.height = input.read_u32::<LE>()?;
        map.reference_point = (input.read_u32::<LE>()?, input.read_u32::<LE>()?);
        map.unchangeable_limit = input.read_u8()?;
        map.values = vec![0; (map.width * map.height) as usize];
        input.read_i8_into(&mut map.values)?;
        Ok(map)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct QuadrantLog {
    pub explored_tiles: u32,
    pub attacks_on_us: u32,
    pub attacks_by_us: u32,
}

impl QuadrantLog {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let explored_tiles = input.read_u32::<LE>()?;
        let attacks_on_us = input.read_u32::<LE>()?;
        let attacks_by_us = input.read_u32::<LE>()?;
        Ok(Self {
            explored_tiles,
            attacks_on_us,
            attacks_by_us,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct InformationAI {
    pub resource_types: Vec<Vec<u32>>,
    pub doctrine: i32,
    pub random_number: i32,
    pub goals: Vec<i32>,
    pub map_size: (u32, u32),
    pub building_lots: Vec<BuildingLot>,
    pub perimeter_walls: (PerimeterWall, PerimeterWall),
    pub attack_memories: Vec<AttackMemory>,
    pub resource_memories: [Vec<ResourceMemory>; 4],
    pub important_objects: Vec<ImportantObjectMemory>,
    pub important_object_ids: Vec<ObjectID>,
    pub important_unit_ids: Vec<ObjectID>,
    pub important_misc_ids: Vec<ObjectID>,
    pub items_to_defend: Vec<ObjectID>,
    pub player_buildings: Vec<ObjectID>,
    pub player_objects: Vec<ObjectID>,
    pub object_counts: Vec<u32>,
    pub path_map: InfluenceMap,
    pub quadrant_log: [QuadrantLog; 4],
}

impl InformationAI {
    #[allow(clippy::cognitive_complexity)]
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut ai = Self::default();
        for _ in 0..4096 {
            let _garbage = input.read_u32::<LE>()?;
        }
        let mut resource_type_counts = vec![0; 4096];
        input.read_u32_into::<LE>(&mut resource_type_counts)?;
        ai.doctrine = input.read_i32::<LE>()?;
        ai.random_number = input.read_i32::<LE>()?;
        for _ in 0..40 {
            ai.goals.push(input.read_i32::<LE>()?);
        }

        // tribute memory
        for _ in 0..8 {
            for _ in 0..4 {
                let _a = input.read_u32::<LE>()?;
                let _b = input.read_u32::<LE>()?;
            }
        }

        if ai.resource_types.len() < 4 {
            ai.resource_types.resize(4, vec![]);
        }
        for resource_id in 0..4 {
            let num_res = input.read_u32::<LE>()?;
            let list = &mut ai.resource_types[resource_id];
            for _ in 0..num_res {
                list.push(input.read_u32::<LE>()?);
            }
        }

        ai.map_size = (input.read_u32::<LE>()?, input.read_u32::<LE>()?);

        let _last_update_row = input.read_u32::<LE>()?;
        ai.important_objects = {
            let max_important_object_memory = input.read_u32::<LE>()?;
            let mut important_objects = vec![];
            for _ in 0..max_important_object_memory {
                let important_object = ImportantObjectMemory::read_from(&mut input, version)?;
                if important_object.id.is_none() {
                    continue;
                }
                important_objects.push(important_object);
            }
            important_objects
        };

        ai.building_lots = {
            let len = input.read_u32::<LE>()?;
            let mut lots = vec![];
            for _ in 0..len {
                let lot = BuildingLot::read_from(&mut input)?;
                if lot.unit_type_id.is_none() {
                    continue;
                }
                lots.push(lot);
            }
            lots
        };

        ai.perimeter_walls = (
            PerimeterWall::read_from(&mut input, version)?,
            PerimeterWall::read_from(&mut input, version)?,
        );

        ai.attack_memories = {
            let len = input.read_u32::<LE>()?;
            let mut attacks = vec![];
            for _ in 0..len {
                let attack = AttackMemory::read_from(&mut input)?;
                // if attack.unit_type_id.is_none() {
                //     continue;
                // }
                attacks.push(attack);
            }
            attacks
        };

        ai.important_object_ids = read_id_list(&mut input)?;
        ai.important_unit_ids = read_id_list(&mut input)?;
        ai.important_misc_ids = read_id_list(&mut input)?;
        ai.items_to_defend = read_id_list(&mut input)?;
        ai.player_buildings = read_id_list(&mut input)?;
        ai.player_objects = read_id_list(&mut input)?;

        ai.object_counts = {
            let num_counts = if version < 11.51 {
                750
            } else if version < 11.65 {
                850
            } else {
                900
            };

            let mut object_counts = vec![0; num_counts];
            input.read_u32_into::<LE>(&mut object_counts)?;
            object_counts
        };

        let _building_count = input.read_u32::<LE>()?;

        ai.path_map = InfluenceMap::read_from(&mut input)?;
        let _last_wall_position = (input.read_i32::<LE>()?, input.read_i32::<LE>()?);
        let _last_wall_position_2 = (input.read_i32::<LE>()?, input.read_i32::<LE>()?);

        if version < 10.78 {
            input.skip(4 + 4 * 16)?;
        }

        let _save_learn_information = input.read_u32::<LE>()? != 0;
        let _learn_path = {
            let len = input.read_u32::<LE>()?;
            dbg!(len);
            genie_support::read_str(&mut input, len.try_into().unwrap())?
        };

        if version < 11.25 {
            input.skip(0xFF)?;
        }

        ai.quadrant_log = [
            QuadrantLog::read_from(&mut input)?,
            QuadrantLog::read_from(&mut input)?,
            QuadrantLog::read_from(&mut input)?,
            QuadrantLog::read_from(&mut input)?,
        ];

        let _max_resources = [
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
        ];
        let num_resources = [
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
        ];

        ai.resource_memories = {
            let mut resources = [vec![], vec![], vec![], vec![]];
            for (list, &num) in resources.iter_mut().zip(num_resources.iter()) {
                list.reserve(num as usize);
                for _ in 0..num {
                    list.push(ResourceMemory::read_from(&mut input, version)?);
                }
            }
            resources
        };

        let _dropsites_by_age = {
            let mut outer = vec![];
            for _ in 0..4 {
                let mut inner = [0; 4];
                input.read_u32_into::<LE>(&mut inner)?;
                outer.push(inner);
            }
            outer
        };
        let _closest_dropsite = [
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
        ];
        let _closest_dropsite_resource_id = [
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
            input.read_u32::<LE>()?,
        ];
        let _found_forest_tiles = input.read_u32::<LE>()?;

        if version < 10.85 {
            input.skip(64_000)?;
        }

        if version >= 10.90 {
            let mut relics_victory = [0; 9];
            input.read_exact(&mut relics_victory)?;
            let mut wonder_victory = [0; 9];
            input.read_exact(&mut wonder_victory)?;
        }

        if version >= 10.94 {
            let should_farm = input.read_u32::<LE>()?;
            let have_seen_forage = input.read_u32::<LE>()?;
            let have_seen_gold = input.read_u32::<LE>()?;
            let have_seen_stone = input.read_u32::<LE>()?;
            dbg!(
                should_farm,
                have_seen_forage,
                have_seen_gold,
                have_seen_stone,
            );
        }
        if version >= 10.95 {
            let have_seen_forest = input.read_u32::<LE>()?;
            dbg!(have_seen_forest);
        }

        if version > 10.99 {
            let last_player_count_refresh_time = input.read_u32::<LE>()?;
            dbg!(last_player_count_refresh_time);
        }

        let player_unit_counts_size = if version >= 11.51 { 120 } else { 102 };
        let mut player_unit_counts = vec![vec![0; player_unit_counts_size as usize]; 8];
        for unit_counts in player_unit_counts.iter_mut() {
            input.read_u32_into::<LE>(unit_counts)?;
        }

        if version >= 11.09 {
            let mut player_total_building_counts = [0; 8];
            input.read_u32_into::<LE>(&mut player_total_building_counts)?;

            let mut player_real_total_building_counts = [0; 8];
            if version >= 11.21 {
                input.read_u32_into::<LE>(&mut player_real_total_building_counts)?;
            }

            let mut player_total_unit_counts = [0; 8];
            input.read_u32_into::<LE>(&mut player_total_unit_counts)?;

            println!(
                "{:x?} {:x?} {:x?}",
                player_total_building_counts,
                player_real_total_building_counts,
                player_total_unit_counts
            );
        }

        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ResourceAI {
    pub num_resources: u32,
}

impl ResourceAI {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let num_resources = input.read_u32::<LE>()?;
        Ok(Self { num_resources })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_u32::<LE>(self.num_resources)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct StrategyAI {
    pub current_victory_condition: u32,
    pub target_id: u32,
    pub second_target_id: u32,
    pub second_target_type: u32,
    pub target_point: Waypoint,
    pub target_point_2: Waypoint,
    pub target_attribute: u32,
    pub target_number: u32,
    pub victory_condition_change_timeout: u32,
    pub ruleset_name: Option<String>,
    pub vc_ruleset: Vec<u32>,
    pub executing_rules: Vec<u32>,
    pub idle_rules: Vec<u32>,
    pub expert_list_id: Option<u32>,
}

impl StrategyAI {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut ai = Self::default();
        ai.current_victory_condition = input.read_u32::<LE>()?;
        ai.target_id = input.read_u32::<LE>()?;
        ai.second_target_id = input.read_u32::<LE>()?;
        ai.second_target_type = input.read_u32::<LE>()?;
        ai.target_point = Waypoint::read_from(&mut input)?;
        ai.target_point_2 = Waypoint::read_from(&mut input)?;
        ai.target_attribute = input.read_u32::<LE>()?;
        ai.target_number = input.read_u32::<LE>()?;
        ai.victory_condition_change_timeout = input.read_u32::<LE>()?;
        ai.ruleset_name = {
            let len = input.read_u32::<LE>()?;
            genie_support::read_str(&mut input, len.try_into().unwrap())?
        };

        ai.vc_ruleset = read_id_list(&mut input)?;
        ai.executing_rules = read_id_list(&mut input)?;
        ai.idle_rules = read_id_list(&mut input)?;
        if version >= 9.71 {
            ai.expert_list_id = Some(input.read_u32::<LE>()?);
        }

        Ok(ai)
    }

    pub fn write_to(&self, _output: impl Write) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct TacticalAI {
    pub civilians: Vec<ObjectID>,
    pub civilian_explorers: Vec<ObjectID>,
    pub soldiers: Vec<ObjectID>,
    pub ungrouped_soldiers: Vec<ObjectID>,
    pub boats: Vec<ObjectID>,
    pub war_boats: Vec<ObjectID>,
    pub fishing_boats: Vec<ObjectID>,
    pub trade_boats: Vec<ObjectID>,
    pub transport_boats: Vec<ObjectID>,
    pub artifacts: Vec<ObjectID>,
    pub trade_carts: Vec<ObjectID>,
    pub strategic_numbers: Vec<i32>,
    pub players_to_attack: Vec<u32>,
    pub players_to_defend: Vec<u32>,
    pub working_area: Vec<u32>,
    pub units_tasked_this_update: Vec<ObjectID>,
    pub gatherer_distribution: [u32; 4],
    pub desired_gatherer_distribution: [u32; 4],
    pub resources_needed: [u32; 4],
    pub group_id: u32,
    pub groups: Vec<()>,
}

impl TacticalAI {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut ai = Self::default();

        ai.civilians = read_id_list(&mut input)?;
        ai.civilian_explorers = read_id_list(&mut input)?;

        let num_gatherers = input.read_u32::<LE>()?;
        let desired_num_gatherers = input.read_u32::<LE>()?;

        // more stuff here

        ai.soldiers = read_id_list(&mut input)?;
        ai.ungrouped_soldiers = read_id_list(&mut input)?;
        ai.boats = read_id_list(&mut input)?;
        ai.war_boats = read_id_list(&mut input)?;
        ai.fishing_boats = read_id_list(&mut input)?;
        ai.trade_boats = read_id_list(&mut input)?;
        ai.transport_boats = read_id_list(&mut input)?;
        ai.artifacts = read_id_list(&mut input)?;
        ai.trade_carts = read_id_list(&mut input)?;

        todo!()
    }

    pub fn write_to(&self, _output: impl Write, _version: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct PlayerAI {
    pub main_ai: MainAI,
    pub build_ai: BuildAI,
    pub construction_ai: ConstructionAI,
    pub diplomacy_ai: DiplomacyAI,
    pub emotional_ai: EmotionalAI,
    pub information_ai: InformationAI,
    pub resource_ai: ResourceAI,
    pub strategy_ai: StrategyAI,
}

impl PlayerAI {
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let main_ai = MainAI::read_from(&mut input)?;
        let build_ai = BuildAI::read_from(&mut input, version)?;
        let construction_ai = ConstructionAI::read_from(&mut input, version)?;
        let diplomacy_ai = DiplomacyAI::read_from(&mut input)?;
        let emotional_ai = EmotionalAI::read_from(&mut input)?;
        let information_ai = InformationAI::read_from(&mut input, version)?;
        let resource_ai = ResourceAI::read_from(&mut input)?;
        let strategy_ai = StrategyAI::read_from(&mut input, version)?;

        Ok(Self {
            main_ai,
            build_ai,
            construction_ai,
            diplomacy_ai,
            emotional_ai,
            information_ai,
            resource_ai,
            strategy_ai,
        })
    }

    pub fn write_to(&self, _output: impl Write, _version: f32) -> Result<()> {
        todo!()
    }
}

fn read_id_list<T: From<u32>>(mut input: impl Read) -> Result<Vec<T>> {
    let len = input.read_u32::<LE>()?;
    let mut ids = vec![];
    for _ in 0..len {
        ids.push(input.read_u32::<LE>()?.into());
    }
    Ok(ids)
}
