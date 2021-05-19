//! Provides an Age of Empires series recorded game file reader.
//!
//! ## Version Support
//! This crate can read Age of Empires 1, Age of Empires 2: The Conquerors, and HD Edition recorded game files.
//!
//! ## Credits
//! Most of the `.mgl`, `.mgx`, `.mgz` format specification was taken from Bari's classic [mgx
//! format description][], the [recage][] Node.js library, and Happyleaves' [aoc-mgz][] Python library.
//!
//! [mgx format description]: https://web.archive.org/web/20090215065209/http://members.at.infoseek.co.jp/aocai/mgx_format.html
//! [recage]: https://github.com/genie-js/recage
//! [aoc-mgz]: https://github.com/happyleavesaoc/aoc-mgz

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
// #![warn(missing_docs)]
#![warn(unused)]

pub mod actions;
pub mod ai;
pub mod element;
pub mod header;
pub mod map;
pub mod player;
pub mod reader;
pub mod string_table;
pub mod unit;
pub mod unit_action;
pub mod unit_type;
pub mod version;

use crate::actions::{Action, Meta};
use crate::element::ReadableElement;
use crate::reader::{RecordingHeaderReader, SmallBufReader};
use crate::Difficulty::{Easiest, Extreme, Hard, Hardest, Moderate, Standard};
use byteorder::{ReadBytesExt, LE};
use flate2::bufread::DeflateDecoder;
use genie_scx::DLCOptions;
use genie_support::{fallible_try_from, fallible_try_into, infallible_try_into};
pub use header::Header;
use std::fmt::Debug;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
pub use version::*;

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

///
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Got a sync message, but the log header said there would be a sync message {0} ticks later. The recorded game file may be corrupt")]
    UnexpectedSync(u32),
    #[error("Expected a sync message at this point, the recorded game file may be corrupt")]
    ExpectedSync,
}

/// Errors that may occur while reading a recorded game file.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    SyncError(#[from] SyncError),
    #[error(transparent)]
    DecodeStringError(#[from] genie_support::DecodeStringError),
    #[error("Could not read embedded scenario data: {0}")]
    ReadScenarioError(#[from] genie_scx::Error),
    #[error("Failed to parse DE JSON chat message: {0}")]
    DEChatMessageJsonError(#[from] serde_json::Error),
    #[error(
        "Failed to parse DE JSON chat message, JSON is missing the key {0}, or value is invalid"
    )]
    ParseDEChatMessageError(&'static str),
    #[error(
    "Failed to find static marker in recording (expected {1:#x} ({1}), found {2:#x} ({2}), version {0}, {3}:{4}, found next {1:#x} ({1}) {5} bytes further)"
    )]
    MissingMarker(f32, u128, u128, &'static str, u32, u64),
    #[error("Failed parsing header at position {0}: {1}")]
    HeaderError(u64, Box<Error>),
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
pub struct BodyActions<R>
where
    R: BufRead,
{
    data_version: f32,
    input: R,
    meta: Meta,
    remaining_syncs_until_checksum: u32,
}

impl<R> BodyActions<R>
where
    R: BufRead,
{
    pub fn new(mut input: R, data_version: f32) -> Result<Self> {
        let meta = if data_version >= 11.76 {
            Meta::read_from_mgx(&mut input)?
        } else {
            Meta::read_from_mgl(&mut input)?
        };
        let remaining_syncs_until_checksum = meta.checksum_interval;
        Ok(Self {
            data_version,
            input,
            meta,
            remaining_syncs_until_checksum,
        })
    }
}

impl<R> Iterator for BodyActions<R>
where
    R: BufRead,
{
    type Item = Result<Action>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.meta.use_sequence_numbers {
            let _sequence = match self.input.read_u8() {
                Ok(s) => s,
                Err(err) => return Some(Err(err.into())),
            };
        }
        match self.input.read_i32::<LE>() {
            Ok(0x00) => {
                if self.remaining_syncs_until_checksum == 0 {
                    self.remaining_syncs_until_checksum = self.meta.checksum_interval;
                    Some(actions::Sync::read_from(&mut self.input).map(Action::Sync))
                } else {
                    Some(Err(SyncError::UnexpectedSync(
                        self.remaining_syncs_until_checksum,
                    )
                    .into()))
                }
            }
            Ok(0x01) => Some(actions::Command::read_from(&mut self.input).map(Action::Command)),
            Ok(0x02) => {
                match self.remaining_syncs_until_checksum.checked_sub(1) {
                    Some(n) => self.remaining_syncs_until_checksum = n,
                    None => return Some(Err(SyncError::ExpectedSync.into())),
                }
                Some(actions::Time::read_from(&mut self.input).map(Action::Time))
            }
            Ok(0x03) => Some(actions::ViewLock::read_from(&mut self.input).map(Action::ViewLock)),
            Ok(0x04) => Some(actions::Chat::read_from(&mut self.input).map(Action::Chat)),
            // AoE2:DE however (also) uses the op field as length field
            Ok(length) if self.data_version >= DE_SAVE_VERSION => {
                let mut buffer = vec![0u8; length as usize];
                match self.input.read_exact(&mut buffer) {
                    Ok(_) => {
                        Some(actions::EmbeddedAction::from_buffer(buffer).map(Action::Embedded))
                    }
                    Err(err) => Some(Err(err.into())),
                }
            }
            Ok(id) => panic!("unsupported action type {:#x}", id),
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => None,
            Err(err) => Some(Err(err.into())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Difficulty {
    Easiest,
    // ???
    Easy,
    Standard,
    Moderate,
    Hard,
    Hardest,
    /// Age of Empires 2: Definitive Edition only.
    Extreme,
}

impl From<u32> for Difficulty {
    fn from(val: u32) -> Self {
        match val {
            0 => Hardest,
            1 => Hard,
            2 => Moderate,
            3 => Standard,
            4 => Easiest,
            5 => Extreme,
            _ => unimplemented!("Don't know any difficulty with value {}", val),
        }
    }
}

impl Default for Difficulty {
    fn default() -> Self {
        Difficulty::Standard
    }
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
    #[allow(unused)]
    next_header: Option<u64>,
    #[allow(unused)]
    game_version: GameVersion,
    save_version: f32,
}

impl<R> RecordedGame<R>
where
    R: Read + Seek,
{
    pub fn new(mut input: R) -> Result<Self> {
        let file_size = {
            input.seek(SeekFrom::Start(0))?;
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
            let version_reader = SmallBufReader::new(&mut input);
            let mut deflate = DeflateDecoder::new(version_reader);
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

    pub fn save_version(&self) -> f32 {
        self.save_version
    }

    fn seek_to_first_header(&mut self) -> Result<()> {
        self.inner.seek(SeekFrom::Start(self.header_start))?;

        Ok(())
    }

    pub fn get_header_data(&mut self) -> Result<Vec<u8>> {
        let mut header = vec![];
        let mut deflate = self.get_header_deflate()?;
        deflate.read_to_end(&mut header)?;
        Ok(header)
    }

    fn seek_to_body(&mut self) -> Result<()> {
        self.inner.seek(SeekFrom::Start(self.header_end))?;

        Ok(())
    }

    pub fn get_header_deflate(&mut self) -> Result<DeflateDecoder<io::Take<BufReader<&mut R>>>> {
        self.seek_to_first_header()?;
        let reader = BufReader::new(&mut self.inner).take(self.header_end - self.header_start);
        Ok(DeflateDecoder::new(reader))
    }

    pub fn header(&mut self) -> Result<Header> {
        let deflate = self.get_header_deflate()?;
        let mut reader = RecordingHeaderReader::new(deflate);
        let header = Header::read_from(&mut reader).map_err(|err| match err {
            Error::HeaderError(pos, err) => Error::HeaderError(pos, err),
            err => Error::HeaderError(reader.position() as u64, Box::new(err)),
        })?;
        Ok(header)
    }

    pub fn actions(&mut self) -> Result<BodyActions<BufReader<&mut R>>> {
        self.seek_to_body()?;
        BodyActions::new(BufReader::new(&mut self.inner), self.save_version)
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
    #[should_panic = "AI data cannot be fully parsed"]
    fn incomplete_up_15_rec_with_ai() {
        let f = File::open("test/rec.20181208-195117.mgz").unwrap();
        let mut r = RecordedGame::new(f).unwrap();
        r.header().expect("AI data cannot be fully parsed");
        for act in r.actions().unwrap() {
            let _act = act.unwrap();
        }
    }

    #[test]
    fn aoc_1_0_rec() -> anyhow::Result<()> {
        let f = File::open("test/missyou_finally_vs_11.mgx")?;
        let mut r = RecordedGame::new(f)?;
        r.header()?;
        for act in r.actions()? {
            let _ = act?;
        }
        Ok(())
    }

    #[test]
    fn aok_rec() -> anyhow::Result<()> {
        let f = File::open("test/aok.mgl")?;
        let mut r = RecordedGame::new(f)?;
        r.header()?;
        for act in r.actions()? {
            let _ = act?;
        }
        Ok(())
    }

    #[test]
    fn aoe2de_rec() -> anyhow::Result<()> {
        let f = File::open("test/AgeIIDE_Replay_90000059.aoe2record")?;
        let mut r = RecordedGame::new(f)?;
        println!("aoe2de save version {}", r.save_version);
        let _header = r.header()?;
        for act in r.actions()? {
            let _ = act?;
        }
        Ok(())
    }

    #[test]
    fn aoe2de_2_rec() -> anyhow::Result<()> {
        let f = File::open("test/AgeIIDE_Replay_90889731.aoe2record")?;
        let mut r = RecordedGame::new(f)?;
        println!("aoe2de save version {}", r.save_version);
        let _header = r.header()?;
        for act in r.actions()? {
            let _ = act?;
        }
        Ok(())
    }
}
