use crate::ObjectID;
use crate::Result;
use std::convert::TryInto;
use std::io::{Read, Write};
use byteorder::{LE, ReadBytesExt, WriteBytesExt};

#[derive(Debug, Clone)]
pub struct Player {
}

impl Player {
    pub fn read_from(mut input: impl Read, version: f32, num_players: u8) -> Result<Self> {
        let ty = input.read_u8()?;
        if version >= 10.55 {
            assert_eq!(input.read_u8()?, 11);
        }
        let mut relations = vec![0; usize::from(num_players)];
        for r in relations.iter_mut() {
            *r = input.read_u8()?;
        }
        let mut diplomacy = vec![0; 9];
        for r in diplomacy.iter_mut() {
            *r = input.read_u32::<LE>()?;
        }
        let allied_los = input.read_u32::<LE>()? != 0;
        let allied_victory = input.read_u8()? != 0;
        let name_len = input.read_u16::<LE>()?;
        let name = genie_support::read_str(&mut input, usize::from(name_len))?;
        if version >= 10.55 {
            assert_eq!(input.read_u8()?, 22);
        }
        let num_attributes = input.read_u32::<LE>()?;
        if version >= 10.55 {
            assert_eq!(input.read_u8()?, 33);
        }
        let mut attributes = vec![0.0; num_attributes.try_into().unwrap()];
        for v in attributes.iter_mut() {
            *v = input.read_f32::<LE>()?;
        }
        if version >= 10.55 {
            assert_eq!(input.read_u8()?, 11);
        }
        let view = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        let num_saved_views = input.read_i32::<LE>()?;
        // saved view count can be negative
        let mut saved_views = vec![(0.0, 0.0); num_saved_views.try_into().unwrap_or(0)];
        for sv in saved_views.iter_mut() {
            *sv = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
        }
        let spawn_location = (input.read_u16::<LE>()?, input.read_u16::<LE>()?);
        let culture = input.read_u8()?;
        let civilization_id = input.read_u8()?;
        let game_status = input.read_u8()?;
        let resigned = input.read_u8()? != 0;
        if version >= 10.55 {
            assert_eq!(input.read_u8()?, 11);
        }
        let color = input.read_u8()?;
        if version >= 10.55 {
            assert_eq!(input.read_u8()?, 11);
        }
        let pathing_attempt_cap = input.read_u32::<LE>()?;
        let pathing_delay_cap = input.read_u32::<LE>()?;

        // Unit counts
        let counts = if version >= 11.65 {
            (900, 100, 900, 100)
        } else if version >= 11.51 {
            (850, 100, 850, 100)
        } else {
            (750, 100, 750, 100)
        };
        let mut object_categories_count = vec![0; counts.0];
        for count in object_categories_count.iter_mut() {
            *count = input.read_u16::<LE>()?;
        }
        let mut object_groups_count = vec![0; counts.1];
        for count in object_groups_count.iter_mut() {
            *count = input.read_u16::<LE>()?;
        }

        let mut built_object_categories_count = vec![0; counts.2];
        for count in built_object_categories_count.iter_mut() {
            *count = input.read_u16::<LE>()?;
        }
        let mut built_object_groups_count = vec![0; counts.3];
        for count in built_object_groups_count.iter_mut() {
            *count = input.read_u16::<LE>()?;
        }

        let total_units_count = input.read_u16::<LE>()?;
        let total_buildings_count = input.read_u16::<LE>()?;
        let built_units_count = input.read_u16::<LE>()?;
        let built_buildings_count = input.read_u16::<LE>()?;

        // formations
        let line_ratio = input.read_u32::<LE>()?;
        let column_ratio = input.read_u32::<LE>()?;
        let min_column_distance = input.read_u32::<LE>()?;
        let column_to_line_distance = input.read_u32::<LE>()?;
        let auto_formations = input.read_u32::<LE>()?;
        let formations_influence_distance = input.read_f32::<LE>()?;
        let break_auto_formations_by_speed = if version >= 10.81 {
            input.read_f32::<LE>()?
        } else {
            0.0
        };

        // Escrow
        let pending_debits = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        let escrow_amounts = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        let escrow_percents = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );

        // view scrolling
        if version >= 10.51 {
            let scroll_vector = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
            let scroll_end = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
            let scroll_start = (input.read_f32::<LE>()?, input.read_f32::<LE>()?);
            let scroll_total_distance = input.read_f32::<LE>()?;
            let scroll_distance = input.read_f32::<LE>()?;
        }

        // AI state
        if version >= 11.45 {
            let easiest_reaction_percent = input.read_f32::<LE>()?;
            let easier_reaction_percent = input.read_f32::<LE>()?;
            let task_ungrouped_soldiers = input.read_u8()? != 0;
        }

        // selected units
        if version >= 11.72 {
            let num_selections = input.read_u32::<LE>()?;
            let selection = if num_selections > 0 {
                let object_id: ObjectID = input.read_u32::<LE>()?.into();
                let object_properties = input.read_u32::<LE>()?;
                let mut selected_ids = vec![ObjectID(0); num_selections.try_into().unwrap()];
                for id in selected_ids.iter_mut() {
                    *id = input.read_u32::<LE>()?.into();
                }
                Some((
                    object_id,
                    object_properties,
                    selected_ids,
                ))
            } else {
                None
            };
        }

        if version >= 10.55 {
            assert_eq!(input.read_u8()?, 11);
            assert_eq!(input.read_u8()?, 11);
        }

        let ty = input.read_u8()?;
        let update_count = input.read_u32::<LE>()?;
        let update_count_need_help = input.read_u32::<LE>()?;

        if version >= 10.02 {
            let alerted_enemy_count = input.read_u32::<LE>()?;
            let regular_attack_count = input.read_u32::<LE>()?;
            let regular_attack_mode = input.read_u8()?;
            let regular_attack_location = (
                input.read_f32::<LE>()?,
                input.read_f32::<LE>()?,
            );
            let town_attack_count = input.read_u32::<LE>()?;
            let town_attack_mode = input.read_u8()?;
            let town_attack_location = (
                input.read_f32::<LE>()?,
                input.read_f32::<LE>()?,
            );
        }

        let fog_update = input.read_u32::<LE>()?;
        let update_time = input.read_f32::<LE>()?;

        // if is userpatch
        let up_data = UserPatchData::read_from(&mut input)?;

        let tech = PlayerTech::read_from(&mut input)?;
        dbg!(tech);

        Ok(Player {})
    }
}

#[derive(Debug, Default, Clone)]
pub struct TechState {
    pub progress: f32,
    pub state: i16,
    pub modifiers: (i16, i16, i16),
    pub time_modifier: i16,
}

impl TechState {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut state = Self::default();
        state.progress = input.read_f32::<LE>()?;
        state.state = input.read_i16::<LE>()?;
        state.modifiers = (
            input.read_i16::<LE>()?,
            input.read_i16::<LE>()?,
            input.read_i16::<LE>()?,
        );
        state.time_modifier = input.read_i16::<LE>()?;
        Ok(state)
    }
}

#[derive(Debug, Clone)]
pub struct PlayerTech {
    pub techs: Vec<TechState>,
}

impl PlayerTech {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let num_techs = input.read_u16::<LE>()?;
        let mut techs = Vec::with_capacity(usize::from(num_techs));
        for _ in 0..num_techs {
            techs.push(TechState::read_from(&mut input)?);
        }
        Ok(Self { techs })
    }
}

#[derive(Debug, Clone)]
pub struct UserPatchData {
}

impl UserPatchData {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        {
            let mut bytes = vec![0; 4080];
            input.read_exact(&mut bytes)?;
        }

        let mut category_priorities = vec![0; 900];
        let mut group_priorities = vec![0; 100];

        for val in category_priorities.iter_mut() {
            *val = input.read_u16::<LE>()?;
        }

        for val in group_priorities.iter_mut() {
            *val = input.read_u16::<LE>()?;
        }

        {
            let mut bytes = vec![0; 2096];
            input.read_exact(&mut bytes)?;
        }

        Ok(Self {
        })
    }
}
