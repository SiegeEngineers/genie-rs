use crate::actions::Action;
use byteorder::{ReadBytesExt, LE};
use genie_scx::DLCOptions;
use std::io::{Read, Result, Seek, SeekFrom};

pub type PlayerID = i8;
pub type ObjectID = i32;

pub mod actions;

pub struct BodyActions<'r, R>
where
    R: Read,
{
    input: &'r mut R,
}

impl<'r, R> Iterator for BodyActions<'r, R>
where
    R: Read,
{
    type Item = Action;
    fn next(&mut self) -> Option<Self::Item> {
        Action::read(self.input).map(Some).unwrap_or(None)
    }
}

pub type GameVersion = [u8; 8];
pub struct Header {
    game_version: GameVersion,
    save_version: f32,
}

enum Difficulty {}

enum MapSize {}

enum MapType {}

enum Visibility {}

enum ResourceLevel {}

enum Age {}

enum GameMode {}

enum GameSpeed {}

pub struct HDGameOptions {
    dlc_options: DLCOptions,
    difficulty: Difficulty,
    map_size: MapSize,
    map_type: MapType,
    visibility: Visibility,
    starting_resources: ResourceLevel,
    starting_age: Age,
    ending_age: Age,
    game_mode: GameMode,
    // if version < 1001
    random_map_name: Option<String>,
    // if version < 1001
    scenario_name: Option<String>,
    game_speed: GameSpeed,
    treaty_length: i32,
    population_limit: i32,
    num_players: i32,
    victory_amount: i32,
    trading_enabled: bool,
    team_bonuses_enabled: bool,
    randomize_positions_enabled: bool,
    full_tech_tree_enabled: bool,
    num_starting_units: i8,
    teams_locked: bool,
    speed_locked: bool,
    multiplayer: bool,
    cheats_enabled: bool,
    record_game: bool,
    animals_enabled: bool,
    predators_enabled: bool,
    // if version > 1.16 && version < 1002
    scenario_player_indices: Vec<i32>,
}

pub struct RecordedGame;

impl RecordedGame {
    pub fn skip_header<R>(input: &mut R) -> Result<()>
    where
        R: Read + Seek,
    {
        let header_len = input.read_u32::<LE>()?;
        let _next_header = input.read_u32::<LE>()?;

        input.seek(SeekFrom::Current(header_len as i64))?;

        Ok(())
    }

    pub fn actions<'r, R>(input: &'r mut R) -> BodyActions<'r, R>
    where
        R: Read,
    {
        BodyActions { input }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn it_works() {
        let mut f = File::open("test/rec.20181208-195117.mgz").unwrap();
        RecordedGame::skip_header(&mut f).unwrap();
        for act in RecordedGame::actions(&mut f) {
            dbg!(act);
        }
    }
}
