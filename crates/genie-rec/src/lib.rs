pub mod actions;
pub mod ai;
pub mod header;
pub mod map;
pub mod player;
pub mod string_table;
pub mod unit;
pub mod unit_action;
pub mod unit_type;

use crate::actions::{Action, Meta};
use byteorder::{ReadBytesExt, LE};
use flate2::read::DeflateDecoder;
use genie_scx::DLCOptions;
use genie_support::{fallible_try_from, fallible_try_into, infallible_try_into};
use header::Header;
use std::fmt::{self, Debug, Display};
use std::io::{self, Read, Seek, SeekFrom};

/// ID identifying a player (0-8).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerID(u8);

impl PlayerID {
    /// Player ID for GAIA, the "nature" player.
    pub const GAIA: Self = Self(0);
}

impl From<u8> for PlayerID {
    #[inline]
    fn from(n: u8) -> Self {
        Self(n)
    }
}

impl From<PlayerID> for u8 {
    #[inline]
    fn from(player_id: PlayerID) -> Self {
        player_id.0
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

impl From<u16> for ObjectID {
    #[inline]
    fn from(n: u16) -> Self {
        Self(n.into())
    }
}

impl From<ObjectID> for u32 {
    #[inline]
    fn from(n: ObjectID) -> Self {
        n.0
    }
}

fallible_try_from!(ObjectID, i16);
fallible_try_from!(ObjectID, i32);
fallible_try_into!(ObjectID, i16);
fallible_try_into!(ObjectID, i32);

/// The game data version string. In practice, this does not really reflect the game version.
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
    /// Read the game version string from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut game_version = [0; 8];
        input.read_exact(&mut game_version)?;
        Ok(Self(game_version))
    }
}

/// Errors that may occur while reading a recorded game file.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    DecodeStringError(#[from] genie_support::DecodeStringError),
}

impl From<genie_support::ReadStringError> for Error {
    fn from(err: genie_support::ReadStringError) -> Self {
        match err {
            genie_support::ReadStringError::DecodeStringError(inner) => inner.into(),
            genie_support::ReadStringError::IoError(inner) => inner.into(),
        }
    }
}

/// Result type alias with `genie_rec::Error` as the error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Iterator over body actions.
pub struct BodyActions<'r, R>
where
    R: Read,
{
    input: &'r mut R,
    version: f32,
    meta: Meta,
    remaining_syncs_until_checksum: u32,
}

impl<'r, R> BodyActions<'r, R>
where
    R: Read,
{
    pub fn new(mut input: &'r mut R, version: f32) -> Result<Self> {
        let meta = if version >= 11.97 {
            // mgx and later have an identifying byte here.
            assert_eq!(input.read_u32::<LE>()?, 4);
            Meta::read_from_mgx(&mut input)?
        } else {
            Meta::read_from_mgl(&mut input)?
        };
        let remaining_syncs_until_checksum = meta.checksum_interval;
        Ok(Self {
            input,
            version,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Difficulty {
    Easiest,
    Easy,
    Standard,
    Hard,
    Hardest,
    /// Age of Empires 2: Definitive Edition only.
    Extreme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MapSize {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MapType {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visibility {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceLevel {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Age {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameMode {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameSpeed {}

#[derive(Debug, Clone)]
pub struct HDGameOptions {
    pub dlc_options: DLCOptions,
    pub difficulty: Difficulty,
    pub map_size: MapSize,
    pub map_type: MapType,
    pub visibility: Visibility,
    pub starting_resources: ResourceLevel,
    pub starting_age: Age,
    pub ending_age: Age,
    pub game_mode: GameMode,
    // if version < 1001
    pub random_map_name: Option<String>,
    // if version < 1001
    pub scenario_name: Option<String>,
    pub game_speed: GameSpeed,
    pub treaty_length: i32,
    pub population_limit: i32,
    pub num_players: i32,
    pub victory_amount: i32,
    pub trading_enabled: bool,
    pub team_bonuses_enabled: bool,
    pub randomize_positions_enabled: bool,
    pub full_tech_tree_enabled: bool,
    pub num_starting_units: i8,
    pub teams_locked: bool,
    pub speed_locked: bool,
    pub multiplayer: bool,
    pub cheats_enabled: bool,
    pub record_game: bool,
    pub animals_enabled: bool,
    pub predators_enabled: bool,
    // if version > 1.16 && version < 1002
    pub scenario_player_indices: Vec<i32>,
}

/// Recorded game reader.
pub struct RecordedGame<R>
where
    R: Read + Seek,
{
    inner: R,
    /// Offset of the main compressed header.
    header_start: u64,
    /// Size of the compressed header.
    header_end: u64,
    /// Offset of the next header, for saved chapters.
    next_header: Option<u64>,
    game_version: GameVersion,
    save_version: f32,
}

impl<R> RecordedGame<R>
where
    R: Read + Seek,
{
    pub fn new(mut input: R) -> Result<Self> {
        let file_size = {
            let size = input.seek(SeekFrom::End(0))?;
            input.seek(SeekFrom::Start(0))?;
            size
        };

        let header_end = u64::from(input.read_u32::<LE>()?);
        let next_header = u64::from(input.read_u32::<LE>()?);

        let header_start = if next_header > file_size { 4 } else { 8 };

        let next_header = if next_header > 0 && next_header < file_size {
            Some(next_header)
        } else {
            None
        };

        let (game_version, save_version) = {
            input.seek(SeekFrom::Start(header_start))?;
            let mut deflate = DeflateDecoder::new(&mut input);
            let game_version = GameVersion::read_from(&mut deflate)?;
            let save_version = deflate.read_f32::<LE>()?;
            (game_version, save_version)
        };

        Ok(Self {
            inner: input,
            header_start,
            header_end,
            next_header,
            game_version,
            save_version,
        })
    }

    fn seek_to_first_header(&mut self) -> Result<()> {
        self.inner.seek(SeekFrom::Start(self.header_start))?;

        Ok(())
    }

    fn seek_to_body(&mut self) -> Result<()> {
        self.inner.seek(SeekFrom::Start(self.header_end))?;

        Ok(())
    }

    pub fn header(&mut self) -> Result<Header> {
        self.seek_to_first_header()?;
        let reader = (&mut self.inner).take(self.header_end - self.header_start);
        let deflate = DeflateDecoder::new(reader);
        let header = Header::read_from(deflate)?;
        Ok(header)
    }

    pub fn actions(&mut self) -> Result<BodyActions<R>> {
        self.seek_to_body()?;
        BodyActions::new(&mut self.inner, self.save_version)
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
    // AI data parsing is incomplete: remove this attribute when the test starts passing
    #[should_panic]
    fn incomplete_up_15_rec_with_ai() {
        let f = File::open("test/rec.20181208-195117.mgz").unwrap();
        let mut r = RecordedGame::new(f).unwrap();
        r.header().expect("AI data cannot be fully parsed");
        for act in r.actions().unwrap() {
            let _act = act.unwrap();
        }
    }

    #[test]
    fn aok_rec() -> anyhow::Result<()> {
        let f = File::open("test/aok.mgl")?;
        let mut r = RecordedGame::new(f)?;
        r.header()?;
        for act in r.actions()? {
            println!("{:?}", act?);
        }
        Ok(())
    }
}
