use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Read, Write};
use genie_support::read_opt_u16;

/// A map tile.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tile {
    /// The terrain.
    pub terrain: u8,
    /// Terrain type layered on top of this tile, if any.
    pub layered_terrain: Option<u16>,
    /// The elevation level.
    pub elevation: i8,
    /// Unused?
    pub zone: i8,
    /// Definitive Edition 2 value, not sure what it does, only seen `-1` in the wild so far
    mask_type: Option<u16>,
}

impl Tile {
    /// Read a tile from an input stream.
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let mut tile = Self {
            terrain: input.read_u8()?,
            layered_terrain: None,
            elevation: input.read_i8()?,
            zone: input.read_i8()?,
            mask_type: None,
        };
        if version >= 1.28 {
            tile.mask_type = read_opt_u16(&mut input)?;
            tile.layered_terrain = read_opt_u16(&mut input)?;
        }
        Ok(tile)
    }

    /// Write a tile to an output stream.
    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        output.write_u8(self.terrain)?;
        output.write_i8(self.elevation)?;
        output.write_i8(self.zone)?;

        if version >= 1.28 {
            output.write_u16::<LE>(self.mask_type.unwrap_or(0xFFFF))?;
            output.write_u16::<LE>(self.layered_terrain.unwrap_or(0xFFFF))?;
        }

        Ok(())
    }
}

/// Describes the terrain in a map.
#[derive(Debug, Clone)]
pub struct Map {
    /// Width of this map in tiles.
    width: u32,
    /// Height of this map in tiles.
    height: u32,
    /// Matrix of tiles on this map.
    tiles: Vec<Tile>,
}

impl Map {
    /// Create a new, empty map.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            tiles: vec![Default::default(); (width * height) as usize],
        }
    }

    /// Fill the map with the given terrain type.
    pub fn fill(&mut self, terrain_type: u8) {
        for tile in self.tiles.iter_mut() {
            tile.terrain = terrain_type;
        }
    }

    /// Read map/terrain data from an input stream.
    pub fn read_from(mut input: impl Read, version: f32) -> Result<Self> {
        let width = input.read_u32::<LE>()?;
        let height = input.read_u32::<LE>()?;
        log::debug!("Map size: {}×{}", width, height);

        if width > 500 || height > 500 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Unexpected map size {}×{}, this is likely a genie-scx bug.",
                    width, height
                ),
            )
            .into());
        }

        let mut tiles = Vec::with_capacity((height * height) as usize);
        for _ in 0..height {
            for _ in 0..width {
                tiles.push(Tile::read_from(&mut input, version)?);
            }
        }

        Ok(Self {
            width,
            height,
            tiles,
        })
    }

    /// Write map/terrain data to an output stream.
    pub fn write_to(&self, mut output: impl Write, version: f32) -> Result<()> {
        output.write_u32::<LE>(self.width)?;
        output.write_u32::<LE>(self.height)?;

        assert_eq!(self.tiles.len(), (self.height * self.width) as usize);

        for tile in &self.tiles {
            tile.write_to(&mut output, version)?;
        }

        Ok(())
    }

    /// Get the width of the map.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height of the map.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get a tile at the given coordinates.
    ///
    /// If the coordinates are out of bounds, returns None.
    pub fn tile(&self, x: u32, y: u32) -> Option<&Tile> {
        self.tiles.get((y * self.width + x) as usize)
    }

    /// Get a mutable reference to the tile at the given coordinates.
    ///
    /// If the coordinates are out of bounds, returns None.
    pub fn tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        self.tiles.get_mut((y * self.width + x) as usize)
    }

    /// Iterate over all the tiles.
    pub fn tiles(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter()
    }

    /// Iterate over all the tiles, with mutable references.
    ///
    /// This is handy if you want to replace terrains throughout the entire map, for example.
    pub fn tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile> {
        self.tiles.iter_mut()
    }

    /// Iterate over all the tiles by row.
    ///
    /// This is handy if you want to iterate over tiles while keeping track of coordinates.
    ///
    /// ## Example
    /// ```rust
    /// # use genie_scx::{Map, Tile};
    /// # let map = Map::new(120, 120);
    /// let mut ys = vec![];
    /// for (y, row) in map.rows().enumerate() {
    ///     let mut xs = vec![];
    ///     for (x, tile) in row.iter().enumerate() {
    ///         xs.push(x);
    ///     }
    ///     assert_eq!(xs, (0..120).collect::<Vec<usize>>());
    ///     ys.push(y);
    /// }
    /// assert_eq!(ys, (0..120).collect::<Vec<usize>>());
    /// ```
    pub fn rows(&self) -> impl Iterator<Item = &[Tile]> {
        self.tiles.chunks_exact(self.width as usize)
    }

    /// Iterate over all the tiles by row, with mutable references.
    pub fn rows_mut(&mut self) -> impl Iterator<Item = &mut [Tile]> {
        self.tiles.chunks_exact_mut(self.width as usize)
    }
}
