mod color_table;
mod sound;
mod sprite;
mod tech;
mod terrain;

use byteorder::{ReadBytesExt, LE};
pub use color_table::ColorTable;
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
pub use sound::{Sound, SoundItem};
pub use sprite::{SoundProp, Sprite, SpriteAttackSound, SpriteDelta};
use std::io::{Read, Result, Write};
pub use tech::{Tech, TechEffect};
pub use terrain::{
    Terrain, TerrainAnimation, TerrainBorder, TerrainPassGraphic, TerrainRestriction,
    TerrainSpriteFrame, TileSize,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Version([u8; 8]);

impl Version {
    fn from(identifier: [u8; 8]) -> Self {
        Self(identifier)
    }

    pub fn is_swgb(self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct DatFile {
    version: Version,
    pub terrain_tables: Vec<TerrainRestriction>,
    pub tile_sizes: Vec<TileSize>,
    pub terrains: Vec<Terrain>,
    pub terrain_borders: Vec<TerrainBorder>,
    pub color_tables: Vec<ColorTable>,
    pub sounds: Vec<Sound>,
    pub sprites: Vec<Option<Sprite>>,
    pub techs: Vec<Tech>,
}

impl DatFile {
    pub fn from(input: impl Read) -> Result<Self> {
        let mut input = DeflateDecoder::new(input);

        let mut version = [0u8; 8];
        input.read_exact(&mut version)?;
        let version = Version(version);

        let num_terrain_tables = input.read_u16::<LE>()?;
        let num_terrains = input.read_u16::<LE>()?;

        skip(
            &mut input,
            4 * u64::from(num_terrain_tables) + 4 * u64::from(num_terrain_tables),
        )?;

        let mut terrain_tables = vec![];
        for _ in 0..num_terrain_tables {
            terrain_tables.push(TerrainRestriction::from(&mut input, version, num_terrains)?);
        }

        let num_color_tables = input.read_u16::<LE>()?;
        let mut color_tables = vec![];
        for _ in 0..num_color_tables {
            color_tables.push(ColorTable::from(&mut input)?);
        }

        let num_sounds = input.read_u16::<LE>()?;
        let mut sounds = vec![];
        for _ in 0..num_sounds {
            sounds.push(Sound::from(&mut input, version)?);
        }

        let num_sprites = input.read_u16::<LE>()?;
        let mut sprites_exist = vec![];
        for _ in 0..num_sprites {
            sprites_exist.push(input.read_u32::<LE>()? != 0);
        }
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

        let mut terrains = vec![];
        for _ in 0..num_terrains {
            terrains.push(Terrain::from(&mut input, version, num_terrains)?);
        }

        let mut terrain_borders = vec![];
        for _ in 0..16 {
            terrain_borders.push(TerrainBorder::from(&mut input)?);
        }

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

        let num_random_maps = input.read_u32::<LE>()?;
        let _random_maps_pointer = input.read_u32::<LE>()?;

        assert!(num_random_maps == 0, "Random map info is not implemented");

        let num_techs = input.read_u32::<LE>()?;
        let mut techs = vec![];
        for _ in 0..num_techs {
            techs.push(Tech::from(&mut input)?);
        }

        Ok(Self {
            version,
            terrain_tables,
            tile_sizes,
            terrains,
            terrain_borders,
            color_tables,
            sounds,
            sprites,
            techs,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        let mut output = DeflateEncoder::new(output, Compression::default());
        output.write_all(&self.version.0)?;
        Ok(())
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
    fn aoe2() {
        let mut f = File::open("fixtures/aok.dat").unwrap();
        let dat = DatFile::from(&mut f).unwrap();
        dbg!(&dat.techs);
    }
}
