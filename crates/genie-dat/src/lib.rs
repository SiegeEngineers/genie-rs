mod civ;
mod color_table;
mod random_map;
mod sound;
mod sprite;
mod task;
mod tech;
mod terrain;
mod unit_type;

use byteorder::{ReadBytesExt, LE};
pub use civ::{Civilization, CivilizationID};
pub use color_table::ColorTable;
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
pub use random_map::*;
pub use sound::{Sound, SoundID, SoundItem};
pub use sprite::{GraphicID, SoundProp, Sprite, SpriteAttackSound, SpriteDelta, SpriteID};
use std::io::{Read, Result, Write};
pub use task::{Task, TaskList};
pub use tech::{Tech, TechEffect, TechID};
pub use terrain::{
    Terrain, TerrainAnimation, TerrainBorder, TerrainID, TerrainPassGraphic, TerrainRestriction,
    TerrainSpriteFrame, TileSize,
};
pub use unit_type::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameVersion {
    AoK,
    AoC,
    HD,
}

impl GameVersion {
    pub fn as_f32(self) -> f32 {
        use GameVersion::*;
        match self {
            AoK => 11.5,
            AoC => 11.97,
            HD => 12.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FileVersion([u8; 8]);

impl From<[u8; 8]> for FileVersion {
    fn from(identifier: [u8; 8]) -> Self {
        Self(identifier)
    }
}

impl FileVersion {
    /// Is this file built for Star Wars: Galactic Battlegrounds?
    pub fn is_swgb(self) -> bool {
        false
    }

    /// Is this file built for Age of Empires II: The Conquerors?
    pub fn is_aoc(self) -> bool {
        self.into_data_version() == 11.97
    }

    /// Get the data version associated with this file version.
    pub fn into_data_version(self) -> f32 {
        match &self.0 {
            b"VER 5.7\0" => 11.97,
            _ => panic!("unknown version"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatFile {
    file_version: FileVersion,
    game_version: GameVersion,
    pub terrain_tables: Vec<TerrainRestriction>,
    pub tile_sizes: Vec<TileSize>,
    pub terrains: Vec<Terrain>,
    pub terrain_borders: Vec<TerrainBorder>,
    pub color_tables: Vec<ColorTable>,
    pub sounds: Vec<Sound>,
    pub sprites: Vec<Option<Sprite>>,
    pub effects: Vec<TechEffect>,
    pub task_lists: Vec<Option<TaskList>>,
    pub civilizations: Vec<Civilization>,
    pub techs: Vec<Tech>,
}

impl DatFile {
    pub fn from(input: impl Read) -> Result<Self> {
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
        skip(
            &mut input,
            4 * u64::from(num_terrain_tables) + 4 * u64::from(num_terrain_tables),
        )?;

        #[must_use]
        fn read_array<T>(num: usize, mut read: impl FnMut() -> Result<T>) -> Result<Vec<T>> {
            let mut list = vec![];
            for _ in 0..num {
                list.push(read()?);
            }
            Ok(list)
        }

        let terrain_tables = read_array(num_terrain_tables.into(), || {
            TerrainRestriction::from(&mut input, file_version, num_terrains)
        })?;

        let num_color_tables = input.read_u16::<LE>()?;
        let color_tables = read_array(num_color_tables.into(), || ColorTable::from(&mut input))?;

        let num_sounds = input.read_u16::<LE>()?;
        let sounds = read_array(num_sounds.into(), || Sound::from(&mut input, file_version))?;

        let num_sprites = input.read_u16::<LE>()?;
        let sprites_exist = read_array(num_sprites.into(), || {
            input.read_u32::<LE>().map(|n| n != 0)
        })?;
        let mut sprites = vec![];
        for exists in sprites_exist {
            sprites.push(if exists {
                Some(Sprite::from(&mut input)?)
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
            *val = TileSize::from(&mut input)?;
        }

        // Padding
        input.read_i16::<LE>()?;

        let terrains = read_array(num_terrains_fixed.into(), || {
            Terrain::from(&mut input, file_version, num_terrains_fixed)
        })?;
        let terrain_borders = read_array(16, || TerrainBorder::from(&mut input))?;

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
        skip(&mut input, 21 + 157 * 4)?;

        let num_random_maps = input.read_u32::<LE>()? as usize;
        let _random_maps_pointer = input.read_u32::<LE>()?;

        let mut random_maps = read_array(num_random_maps, || RandomMapInfo::from(&mut input))?;
        for map in random_maps.iter_mut() {
            map.finish(&mut input)?;
        }

        let num_effects = input.read_u32::<LE>()? as usize;
        let effects = read_array(num_effects, || TechEffect::from(&mut input))?;

        let num_task_lists = input.read_u32::<LE>()? as usize;
        let task_lists = read_array(num_task_lists, || {
            if input.read_u8()? != 0 {
                TaskList::from(&mut input).map(Some)
            } else {
                Ok(None)
            }
        })?;

        let num_civilizations = input.read_u16::<LE>()?;
        let civilizations = read_array(num_civilizations.into(), || {
            let player_type = input.read_i8()?;
            assert_eq!(player_type, 1);
            Civilization::from(&mut input, game_version)
        })?;

        let num_techs = input.read_u16::<LE>()?;
        let techs = read_array(num_techs.into(), || Tech::from(&mut input))?;

        let _time_slice = input.read_u32::<LE>()?;
        let _unit_kill_rate = input.read_u32::<LE>()?;
        let _unit_kill_total = input.read_u32::<LE>()?;
        let _unit_hit_point_rate = input.read_u32::<LE>()?;
        let _unit_hit_point_total = input.read_u32::<LE>()?;
        let _razing_kill_rate = input.read_u32::<LE>()?;
        let _razing_kill_total = input.read_u32::<LE>()?;

        Ok(Self {
            file_version,
            game_version,
            terrain_tables,
            tile_sizes,
            terrains,
            terrain_borders,
            color_tables,
            sounds,
            sprites,
            effects,
            task_lists,
            civilizations,
            techs,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        let mut output = DeflateEncoder::new(output, Compression::default());
        output.write_all(&self.file_version.0)?;
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

fn skip<R: Read>(input: &mut R, bytes: u64) -> Result<u64> {
    std::io::copy(&mut input.by_ref().take(bytes), &mut std::io::sink())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn aok() {
        let mut f = File::open("fixtures/aok.dat").unwrap();
        let dat = DatFile::from(&mut f).unwrap();
        assert_eq!(dat.civilizations.len(), 14);
    }

    #[test]
    fn aoc() {
        let mut f = File::open("fixtures/aoc1.0c.dat").unwrap();
        let dat = DatFile::from(&mut f).unwrap();
        assert_eq!(dat.civilizations.len(), 19);
    }

    #[test]
    fn hd_edition() {
        let mut f = File::open("fixtures/hd.dat").unwrap();
        let dat = DatFile::from(&mut f).unwrap();
        assert_eq!(dat.civilizations.len(), 32);
    }
}
