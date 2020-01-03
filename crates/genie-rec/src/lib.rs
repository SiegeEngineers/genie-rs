pub mod actions;
pub mod header;
pub mod string_table;

use crate::actions::{Action, Meta};
use byteorder::{ReadBytesExt, LE};
use flate2::read::DeflateDecoder;
use genie_scx::DLCOptions;
use genie_support::{fallible_try_from, infallible_try_into};
use header::Header;
use std::fmt;
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerID(u8);

impl From<u8> for PlayerID {
    #[inline]
    fn from(n: u8) -> Self {
        Self(n)
    }
}

impl From<PlayerID> for u8 {
    #[inline]
    fn from(object_id: PlayerID) -> Self {
        object_id.0
    }
}

fallible_try_from!(PlayerID, i32);
fallible_try_from!(PlayerID, u32);
fallible_try_from!(PlayerID, i16);
fallible_try_from!(PlayerID, u16);
fallible_try_from!(PlayerID, i8);
infallible_try_into!(PlayerID, i16);
infallible_try_into!(PlayerID, u16);
infallible_try_into!(PlayerID, i32);
infallible_try_into!(PlayerID, u32);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectID(u32);

impl From<u32> for ObjectID {
    #[inline]
    fn from(n: u32) -> Self {
        Self(n)
    }
}

fallible_try_from!(ObjectID, i32);
fallible_try_from!(ObjectID, i16);
infallible_try_into!(ObjectID, u32);

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// Iterator over body actions.
pub struct BodyActions<'r, R>
where
    R: Read,
{
    input: &'r mut R,
    meta: Meta,
    remaining_syncs_until_checksum: u32,
}

impl<'r, R> BodyActions<'r, R>
where
    R: Read,
{
    pub fn new(input: &'r mut R) -> Result<Self> {
        assert_eq!(input.read_u32::<LE>()?, 4);
        let meta = Meta::read_from(input)?;
        let remaining_syncs_until_checksum = meta.checksum_interval;
        Ok(Self {
            input,
            meta,
            remaining_syncs_until_checksum,
        })
    }
}

impl<'r, R> Iterator for BodyActions<'r, R>
where
    R: Read,
{
    type Item = Result<Action>;
    fn next(&mut self) -> Option<Self::Item> {
        // TODO return Option<Result> instead
        match self.input.read_i32::<LE>() {
            Ok(0x01) => Some(actions::Command::read_from(self.input).map(Action::Command)),
            Ok(0x02) => {
                self.remaining_syncs_until_checksum -= 1;
                let includes_checksum = self.remaining_syncs_until_checksum == 0;
                if includes_checksum {
                    self.remaining_syncs_until_checksum = self.meta.checksum_interval;
                }
                Some(
                    actions::Sync::read_from(
                        self.input,
                        self.meta.use_sequence_numbers,
                        includes_checksum,
                    )
                    .map(Action::Sync),
                )
            }
            Ok(0x03) => Some(actions::ViewLock::read_from(&mut self.input).map(Action::ViewLock)),
            Ok(0x04) => Some(actions::Chat::read_from(self.input).map(Action::Chat)),
            Ok(id) => panic!("unsupported action type {:#x}", id),
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => None,
            Err(err) => Some(Err(err.into())),
        }
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
    header_len: u64,
    next_header: Option<u64>,
}

impl<R> RecordedGame<R>
where
    R: Read + Seek,
{
    pub fn new(mut input: R) -> Result<Self> {
        let header_len = u64::from(input.read_u32::<LE>()?);
        let next_header = u64::from(input.read_u32::<LE>()?);

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

    fn seek_to_header(&mut self) -> Result<()> {
        self.inner.seek(SeekFrom::Start(8))?;

        Ok(())
    }

    fn seek_to_body(&mut self) -> Result<()> {
        self.inner.seek(SeekFrom::Start(self.header_len))?;

        Ok(())
    }

    pub fn header(&mut self) -> Result<Header> {
        self.seek_to_header()?;
        let reader = (&mut self.inner).take(self.header_len);
        let deflate = DeflateDecoder::new(reader);
        let header = Header::from(deflate)?;
        Ok(header)
    }

    pub fn actions(&mut self) -> Result<BodyActions<R>> {
        self.seek_to_body()?;
        BodyActions::new(&mut self.inner)
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
        for act in r.actions().unwrap() {
            println!("{:?}", act.unwrap());
        }
    }
}
