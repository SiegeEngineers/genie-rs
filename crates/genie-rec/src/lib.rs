pub mod actions;
pub mod header;
pub mod string_table;

use crate::actions::Action;
use byteorder::{ReadBytesExt, LE};
use flate2::read::DeflateDecoder;
use genie_scx::DLCOptions;
use header::Header;
use std::io::{Read, Result, Seek, SeekFrom};

pub type PlayerID = i8;
pub type ObjectID = i32;

/// Iterator over body actions.
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
        // TODO return Option<Result> instead
        Action::read(self.input).ok()
    }
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

pub struct RecordedGame<R>
where
    R: Read + Seek,
{
    inner: R,
    header_len: u32,
    next_header: Option<u32>,
}

impl<R> RecordedGame<R>
where
    R: Read + Seek,
{
    pub fn new(mut input: R) -> Result<Self> {
        let header_len = input.read_u32::<LE>()?;
        let next_header = input.read_u32::<LE>()?;

        Ok(Self {
            inner: input,
            header_len,
            next_header: if next_header == 0 {
                None
            } else {
                Some(next_header)
            },
        })
    }

    fn seek_to_body(&mut self) -> Result<()> {
        self.inner
            .seek(SeekFrom::Start(self.header_len as u64 + 8))?;

        Ok(())
    }

    pub fn header(&mut self) -> Result<Header> {
        let deflate = DeflateDecoder::new(&mut self.inner);
        Header::from(deflate)
    }

    pub fn actions(&mut self) -> BodyActions<R>
    where
        R: Read,
    {
        self.seek_to_body();
        BodyActions {
            input: &mut self.inner,
        }
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn it_works() {
        let f = File::open("test/rec.20181208-195117.mgz").unwrap();
        let mut r = RecordedGame::new(f).unwrap();
        r.header().unwrap();
        for act in r.actions() {
            dbg!(act);
        }
    }
}
