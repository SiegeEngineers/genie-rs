//! A reader for Age of Empires game data files.
//!
//! This crate aims to support every data file that exists, but is for now being tested with AoE1,
//! AoE2, and AoE2: HD Edition.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused)]

mod civ;
mod color_table;
mod random_map;
mod sound;
mod sprite;
mod task;
mod tech;
mod tech_tree;
mod terrain;
mod unit_type;

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
pub use civ::{Civilization, CivilizationID};
pub use color_table::{ColorTable, PaletteIndex};
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use genie_support::{f32_eq, ReadSkipExt, TechID};
pub use random_map::*;
pub use sound::{Sound, SoundID, SoundItem};
pub use sprite::{GraphicID, SoundProp, Sprite, SpriteAttackSound, SpriteDelta, SpriteID};
use std::cmp::{Ordering, PartialOrd};
use std::convert::TryInto;
use std::fmt;
use std::io::{Read, Result, Write};
pub use task::{Task, TaskList};
pub use tech::{Tech, TechEffect};
pub use tech_tree::{
    ParseTechTreeTypeError, TechTree, TechTreeAge, TechTreeBuilding, TechTreeDependencies,
    TechTreeStatus, TechTreeTech, TechTreeType, TechTreeUnit,
};
pub use terrain::{
    Terrain, TerrainAnimation, TerrainBorder, TerrainID, TerrainObject, TerrainPassGraphic,
    TerrainRestriction, TerrainSpriteFrame, TileSize,
};
pub use unit_type::*;

/// A game version targeted by a data file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameVersion {
    /// The original expansion-less Age of Empires 2: Age of Kings.
    AoK,
    /// Age of Empires 2 with the The Conquerors expansion.
    AoC,
    /// Age of EMpires 2: HD Edition.
    HD,
}

impl GameVersion {
    /// Get the most likely internal game data version number for a given game version.
    fn as_f32(self) -> f32 {
        use GameVersion::*;
        match self {
            AoK => 11.5,
            AoC => 11.97,
            HD => 12.0,
        }
    }
}

/// A data file version.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FileVersion([u8; 8]);

impl From<[u8; 8]> for FileVersion {
    fn from(identifier: [u8; 8]) -> Self {
        assert!(matches!(
            identifier,
            // "VER *.*\0"
            [b'V', b'E', b'R', b' ', b'0'..=b'9', b'.', b'0'..=b'9', 0]
        ));
        Self(identifier)
    }
}

impl From<&str> for FileVersion {
    fn from(string: &str) -> Self {
        assert!(string.len() <= 8);
        let mut bytes = [0; 8];
        (&mut bytes[..string.len()]).copy_from_slice(string.as_bytes());
        Self::from(bytes)
    }
}

impl fmt::Display for FileVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(&self.0[0..7]) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "{:?}", self.0),
        }
    }
}

impl PartialOrd for FileVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.major_version().partial_cmp(&other.major_version()) {
            None | Some(Ordering::Equal) => {
                self.minor_version().partial_cmp(&other.minor_version())
            }
            Some(order) => Some(order),
        }
    }
}

impl FileVersion {
    /// Get the major version component, eg the 5 in "VER 5.8".
    fn major_version(self) -> u8 {
        self.0[4] - b'0'
    }
    /// Get the minor version component, eg the 8 in "VER 5.8".
    fn minor_version(self) -> u8 {
        self.0[6] - b'0'
    }

    /// Is this file built for Star Wars: Galactic Battlegrounds?
    pub fn is_swgb(self) -> bool {
        false
    }

    /// Is this file built for Age of Empires II: The Conquerors?
    pub fn is_aoc(self) -> bool {
        let data_version = self.into_data_version();
        f32_eq!(data_version, 11.97)
    }

    /// Is this file built for Age of Empires II: Definitive Edition?
    pub fn is_de2(self) -> bool {
        self >= FileVersion(*b"VER 5.8\0")
    }

    /// Get the data version associated with this file version.
    fn into_data_version(self) -> f32 {
        match &self.0 {
            b"VER 5.7\0" => 11.97,
            _ => panic!("unknown version"),
        }
    }
}

/// A data file.
#[derive(Debug, Clone)]
pub struct DatFile {
    file_version: FileVersion,
    game_version: GameVersion,
    /// Terrain restriction tables.
    pub terrain_tables: Vec<TerrainRestriction>,
    /// Tile size data.
    pub tile_sizes: Vec<TileSize>,
    /// Terrains.
    pub terrains: Vec<Terrain>,
    /// Terrain border data, specifying how different terrains blend.
    pub terrain_borders: Vec<TerrainBorder>,
    /// Random map data from AoE1.
    random_maps: Vec<RandomMapInfo>,
    /// Data about player colours.
    pub color_tables: Vec<ColorTable>,
    /// The available sounds.
    pub sounds: Vec<Sound>,
    /// The available sprites.
    pub sprites: Vec<Option<Sprite>>,
    /// Tech effect data.
    pub effects: Vec<TechEffect>,
    /// Task lists for unit types.
    pub task_lists: Vec<Option<TaskList>>,
    /// The available civilizations.
    pub civilizations: Vec<Civilization>,
    /// Techs or researches.
    pub techs: Vec<Tech>,
    /// Tech tree data.
    pub tech_tree: TechTree,
}

impl DatFile {
    /// Read a data file from a compressed byte stream.
    pub fn read_from(input: impl Read) -> Result<Self> {
        let mut input = DeflateDecoder::new(input);

        let mut file_version = [0u8; 8];
        input.read_exact(&mut file_version)?;
        let file_version = FileVersion(file_version);

        let num_terrain_tables = input.read_u16::<LE>()?;
        let num_terrains = input.read_u16::<LE>()?;

        let game_version = if file_version == FileVersion(*b"VER 5.7\0") {
            match num_terrains {
                32 => GameVersion::AoK,
                41 => GameVersion::AoC,
                100 => GameVersion::HD,
                _ => GameVersion::AoC, // TODO support different versions
            }
        } else {
            GameVersion::AoC // TODO support different versions
        };

        // AoC hardcodes to 42 terrains, but says 41 terrains in the data file.
        // The 42nd terrain is zeroed out.
        let num_terrains_fixed = if game_version == GameVersion::AoC && num_terrains == 41 {
            42
        } else {
            num_terrains
        };

        // Two lists of pointers
        input.skip(4 * u64::from(num_terrain_tables) + 4 * u64::from(num_terrain_tables))?;

        #[must_use]
        fn read_array<T>(num: usize, mut read: impl FnMut() -> Result<T>) -> Result<Vec<T>> {
            let mut list = vec![];
            for _ in 0..num {
                list.push(read()?);
            }
            Ok(list)
        }

        let terrain_tables = read_array(num_terrain_tables.into(), || {
            TerrainRestriction::read_from(&mut input, file_version, num_terrains)
        })?;

        let num_color_tables = input.read_u16::<LE>()?;
        let color_tables = read_array(num_color_tables.into(), || {
            ColorTable::read_from(&mut input)
        })?;

        let num_sounds = input.read_u16::<LE>()?;
        let sounds = read_array(num_sounds.into(), || {
            Sound::read_from(&mut input, file_version)
        })?;

        let num_sprites = input.read_u16::<LE>()?;
        let sprites_exist = read_array(num_sprites.into(), || {
            input.read_u32::<LE>().map(|n| n != 0)
        })?;
        let mut sprites = vec![];
        for exists in sprites_exist {
            sprites.push(if exists {
                Some(Sprite::read_from(&mut input)?)
            } else {
                None
            });
        }

        // Some raw pointer values
        let _map_vtable_pointer = input.read_i32::<LE>()?;
        let _tiles_pointer = input.read_i32::<LE>()?;

        // Bogus stuff
        let _map_width = input.read_i32::<LE>()?;
        let _map_height = input.read_i32::<LE>()?;
        let _world_width = input.read_i32::<LE>()?;
        let _world_height = input.read_i32::<LE>()?;

        let mut tile_sizes = vec![TileSize::default(); 19];
        for val in tile_sizes.iter_mut() {
            *val = TileSize::read_from(&mut input)?;
        }

        // Padding
        input.read_i16::<LE>()?;

        let terrains = read_array(num_terrains_fixed.into(), || {
            Terrain::read_from(&mut input, file_version, num_terrains_fixed)
        })?;
        let terrain_borders = read_array(16, || TerrainBorder::read_from(&mut input))?;

        // Should just skip all this shit probably
        let _map_row_offset = input.read_i32::<LE>()?;
        let _map_min_x = input.read_f32::<LE>()?;
        let _map_min_y = input.read_f32::<LE>()?;
        let _map_max_x = input.read_f32::<LE>()?;
        let _map_max_y = input.read_f32::<LE>()?;
        let _map_max_x = input.read_f32::<LE>()?;
        let _map_max_y = input.read_f32::<LE>()?;
        let _additional_terrain_count = input.read_u16::<LE>()?;
        let _borders_used = input.read_u16::<LE>()?;
        let _max_terrain = input.read_u16::<LE>()?;
        let _tile_width = input.read_u16::<LE>()?;
        let _tile_height = input.read_u16::<LE>()?;
        let _tile_half_width = input.read_u16::<LE>()?;
        let _tile_half_height = input.read_u16::<LE>()?;
        let _elev_height = input.read_u16::<LE>()?;
        let _current_row = input.read_u16::<LE>()?;
        let _current_column = input.read_u16::<LE>()?;
        let _block_begin_row = input.read_u16::<LE>()?;
        let _block_end_row = input.read_u16::<LE>()?;
        let _block_begin_column = input.read_u16::<LE>()?;
        let _block_end_column = input.read_u16::<LE>()?;
        let _seach_map_pointer = input.read_i32::<LE>()?;
        let _seach_map_rows_pointer = input.read_i32::<LE>()?;
        let _any_frame_change = input.read_u8()?;
        let _map_visible = input.read_u8()?;
        let _map_fog_of_war = input.read_u8()?;

        // Lots more pointers and stuff
        input.skip(21 + 157 * 4)?;

        let num_random_maps = input.read_u32::<LE>()? as usize;
        let _random_maps_pointer = input.read_u32::<LE>()?;

        let mut random_maps = read_array(num_random_maps, || RandomMapInfo::read_from(&mut input))?;
        for map in random_maps.iter_mut() {
            map.finish(&mut input)?;
        }

        let num_effects = input.read_u32::<LE>()? as usize;
        let effects = read_array(num_effects, || TechEffect::read_from(&mut input))?;

        let num_task_lists = input.read_u32::<LE>()? as usize;
        let task_lists = read_array(num_task_lists, || {
            if input.read_u8()? != 0 {
                TaskList::read_from(&mut input).map(Some)
            } else {
                Ok(None)
            }
        })?;

        let num_civilizations = input.read_u16::<LE>()?;
        let civilizations = read_array(num_civilizations.into(), || {
            let player_type = input.read_i8()?;
            assert_eq!(player_type, 1);
            Civilization::read_from(&mut input, game_version)
        })?;

        let num_techs = input.read_u16::<LE>()?;
        let techs = read_array(num_techs.into(), || Tech::read_from(&mut input))?;

        let _time_slice = input.read_u32::<LE>()?;
        let _unit_kill_rate = input.read_u32::<LE>()?;
        let _unit_kill_total = input.read_u32::<LE>()?;
        let _unit_hit_point_rate = input.read_u32::<LE>()?;
        let _unit_hit_point_total = input.read_u32::<LE>()?;
        let _razing_kill_rate = input.read_u32::<LE>()?;
        let _razing_kill_total = input.read_u32::<LE>()?;

        let tech_tree = TechTree::read_from(&mut input)?;

        Ok(Self {
            file_version,
            game_version,
            terrain_tables,
            tile_sizes,
            terrains,
            terrain_borders,
            random_maps,
            color_tables,
            sounds,
            sprites,
            effects,
            task_lists,
            civilizations,
            techs,
            tech_tree,
        })
    }

    /// Serialize this data file to an output stream. Compression is applied by this function.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        let num_terrains = if self.game_version == GameVersion::AoC && self.terrains.len() == 42 {
            41
        } else {
            self.terrains.len()
        };

        let mut output = DeflateEncoder::new(output, Compression::default());
        output.write_all(&self.file_version.0)?;
        output.write_u16::<LE>(self.terrain_tables.len().try_into().unwrap())?;
        output.write_u16::<LE>(num_terrains.try_into().unwrap())?;

        // Two lists of pointers
        output.write_all(&vec![
            0u8;
            4 * self.terrain_tables.len()
                + 4 * self.terrain_tables.len()
        ])?;

        for table in &self.terrain_tables {
            table.write_to(
                &mut output,
                self.file_version,
                num_terrains.try_into().unwrap(),
            )?;
        }

        output.write_u16::<LE>(self.color_tables.len().try_into().unwrap())?;
        for table in &self.color_tables {
            table.write_to(&mut output)?;
        }

        output.write_u16::<LE>(self.sounds.len().try_into().unwrap())?;
        for sound in &self.sounds {
            sound.write_to(&mut output, self.file_version)?;
        }

        output.write_u16::<LE>(self.sprites.len().try_into().unwrap())?;
        for maybe_sprite in &self.sprites {
            output.write_u32::<LE>(match maybe_sprite {
                Some(_) => 1,
                None => 0,
            })?;
        }
        for maybe_sprite in &self.sprites {
            if let Some(sprite) = maybe_sprite {
                sprite.write_to(&mut output)?;
            }
        }

        output.write_u32::<LE>(0)?; // map vtable pointer
        output.write_u32::<LE>(0)?; // map tiles pointer
        output.write_u32::<LE>(0)?; // map width
        output.write_u32::<LE>(0)?; // map height
        output.write_u32::<LE>(0)?; // world width
        output.write_u32::<LE>(0)?; // world height

        for size in &self.tile_sizes {
            size.write_to(&mut output)?;
        }

        // Padding
        output.write_i16::<LE>(0)?;

        for terrain in &self.terrains {
            terrain.write_to(&mut output, self.file_version, self.terrains.len() as u16)?;
        }
        for border in &self.terrain_borders {
            border.write_to(&mut output)?;
        }

        // TODO put correct values in
        output.write_i32::<LE>(0)?;
        output.write_f32::<LE>(0.0)?;
        output.write_f32::<LE>(0.0)?;
        output.write_f32::<LE>(0.0)?;
        output.write_f32::<LE>(0.0)?;
        output.write_f32::<LE>(0.0)?;
        output.write_f32::<LE>(0.0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_u16::<LE>(0)?;
        output.write_i32::<LE>(0)?;
        output.write_i32::<LE>(0)?;
        output.write_u8(0)?;
        output.write_u8(0)?;
        output.write_u8(0)?;

        // Lots more pointers and stuff
        let nulls = [0; 21 + 157 * 4];
        output.write_all(&nulls)?;

        output.write_u32::<LE>(self.random_maps.len() as u32)?;
        output.write_u32::<LE>(0)?; // pointer

        for map in &self.random_maps {
            map.write_to(&mut output)?;
        }
        for map in &self.random_maps {
            map.write_commands_to(&mut output)?;
        }

        output.write_u32::<LE>(self.effects.len() as u32)?;
        for effect in &self.effects {
            effect.write_to(&mut output)?;
        }

        output.write_u32::<LE>(self.task_lists.len() as u32)?;
        for task_list in &self.task_lists {
            if let Some(task_list) = task_list {
                output.write_u8(1)?;
                task_list.write_to(&mut output)?;
            } else {
                output.write_u8(0)?;
            }
        }

        output.write_u16::<LE>(self.civilizations.len() as u16)?;
        for civilization in &self.civilizations {
            output.write_i8(1)?; // player type
            civilization.write_to(&mut output, self.game_version)?;
        }

        output.write_u16::<LE>(self.techs.len() as u16)?;
        for tech in &self.techs {
            tech.write_to(&mut output)?;
        }

        output.write_u32::<LE>(0)?;
        output.write_u32::<LE>(0)?;
        output.write_u32::<LE>(0)?;
        output.write_u32::<LE>(0)?;
        output.write_u32::<LE>(0)?;
        output.write_u32::<LE>(0)?;
        output.write_u32::<LE>(0)?;

        self.tech_tree.write_to(&mut output)?;

        Ok(())
    }

    /// Get a tech by its ID.
    pub fn get_tech(&self, id: impl Into<TechID>) -> Option<&Tech> {
        let id: TechID = id.into();
        self.techs.get(usize::from(id))
    }

    /// Get a terrain type by its ID.
    pub fn get_terrain(&self, id: impl Into<TerrainID>) -> Option<&Terrain> {
        let id: TerrainID = id.into();
        self.terrains.get(usize::from(id))
    }

    /// Get the GAIA civilization.
    pub fn get_gaia(&self) -> Option<&Civilization> {
        self.get_civilization(0)
    }

    /// Get a civilization by its ID.
    pub fn get_civilization(&self, id: impl Into<CivilizationID>) -> Option<&Civilization> {
        let id: CivilizationID = id.into();
        self.civilizations.get(usize::from(id))
    }

    /// Get a sound by its ID.
    pub fn get_sound(&self, id: impl Into<SoundID>) -> Option<&Sound> {
        let id: SoundID = id.into();
        self.sounds.get(usize::from(id))
    }

    /// Get a sprite by its ID.
    pub fn get_sprite(&self, id: impl Into<SpriteID>) -> Option<&Sprite> {
        let id: SpriteID = id.into();
        self.sprites.get(usize::from(id)).and_then(Option::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::hash_map::DefaultHasher,
        fs::File,
        hash::{Hash, Hasher},
        io::Cursor,
    };

    #[test]
    fn aok() -> anyhow::Result<()> {
        let mut f = File::open("fixtures/aok.dat")?;
        let dat = DatFile::read_from(&mut f)?;
        assert_eq!(dat.civilizations.len(), 14);
        Ok(())
    }

    #[test]
    fn aoc() -> anyhow::Result<()> {
        let mut f = File::open("fixtures/aoc1.0c.dat")?;
        let dat = DatFile::read_from(&mut f)?;
        assert_eq!(dat.civilizations.len(), 19);
        Ok(())
    }

    #[test]
    fn non_7bit_ascii_tech_name() -> anyhow::Result<()> {
        let mut f = File::open("fixtures/age-of-chivalry.dat")?;
        let dat = DatFile::read_from(&mut f)?;
        assert_eq!(dat.techs[859].name(), "Székely (enable)");
        Ok(())
    }

    #[test]
    fn hd_edition() -> anyhow::Result<()> {
        let mut f = File::open("fixtures/hd.dat")?;
        let dat = DatFile::read_from(&mut f)?;
        assert_eq!(dat.civilizations.len(), 32);
        Ok(())
    }

    #[test]
    fn reserialize() -> anyhow::Result<()> {
        let original = std::fs::read("fixtures/aoc1.0c.dat")?;
        let mut cursor = Cursor::new(&original);
        let dat = DatFile::read_from(&mut cursor)?;
        let mut serialized = vec![];
        dat.write_to(&mut serialized)?;

        let mut cursor = Cursor::new(&serialized);
        let dat2 = DatFile::read_from(&mut cursor)?;

        let mut orig_hasher = DefaultHasher::new();
        let mut new_hasher = DefaultHasher::new();
        format!("{:?}", dat).hash(&mut orig_hasher);
        format!("{:?}", dat2).hash(&mut new_hasher);
        assert_eq!(orig_hasher.finish(), new_hasher.finish());

        Ok(())
    }
}
