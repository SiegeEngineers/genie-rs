use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Result, Write};

#[derive(Debug)]
pub struct Tile {
    /// The terrain.
    pub terrain: i8,
    /// The elevation level.
    pub elevation: i8,
    /// Unused?
    pub zone: i8,
}

#[derive(Debug)]
pub struct Map {
    /// Width of this map in tiles.
    width: u32,
    /// Height of this map in tiles.
    height: u32,
    /// Matrix of tiles on this map.
    tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let width = input.read_u32::<LE>()?;
        let height = input.read_u32::<LE>()?;

        let mut tiles = Vec::with_capacity(height as usize);
        for _ in 0..height {
            let mut row = Vec::with_capacity(width as usize);
            for _ in 0..width {
                row.push(Tile {
                    terrain: input.read_i8()?,
                    elevation: input.read_i8()?,
                    zone: input.read_i8()?,
                });
            }
            tiles.push(row);
        }

        Ok(Self {
            width,
            height,
            tiles,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u32::<LE>(self.width)?;
        output.write_u32::<LE>(self.height)?;

        assert_eq!(self.tiles.len(), self.height as usize);
        for row in &self.tiles {
            assert_eq!(row.len(), self.width as usize);
        }

        for row in &self.tiles {
            for tile in row {
                output.write_i8(tile.terrain)?;
                output.write_i8(tile.elevation)?;
                output.write_i8(tile.zone)?;
            }
        }

        Ok(())
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn tile(&self, x: u32, y: u32) -> Option<&Tile> {
        self.tiles
            .get(y as usize)
            .and_then(|row| row.get(x as usize))
    }

    pub fn tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        self.tiles
            .get_mut(y as usize)
            .and_then(|row| row.get_mut(x as usize))
    }

    pub fn tiles(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter().map(|row| row.iter()).flatten()
    }

    pub fn tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile> {
        self.tiles.iter_mut().map(|row| row.iter_mut()).flatten()
    }
}
