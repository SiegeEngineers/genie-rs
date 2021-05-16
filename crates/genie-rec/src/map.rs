use crate::element::{ReadableHeaderElement, WritableHeaderElement};
use crate::reader::RecordingHeaderReader;
use crate::GameVariant::DefinitiveEdition;
use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::ReadSkipExt;
use std::convert::TryInto;
use std::io::{Read, Write};

/// Data about a map tile.
#[derive(Debug, Default, Clone)]
pub struct Tile {
    /// The terrain type of this tile.
    pub terrain: u8,
    /// The elevation level of this tile.
    pub elevation: u8,
    /// The original terrain type of this tile, if it was later replaced, for example by placing a
    /// Farm. UserPatch 1.5 only.
    pub original_terrain: Option<u8>,
}

impl ReadableHeaderElement for Tile {
    /// Read a tile from an input stream.
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let terrain = input.read_u8()?;
        if input.variant() >= DefinitiveEdition {
            input.skip(1)?;
            let elevation = input.read_u8()?;
            input.skip(4)?;

            // there's another DE version (12.97) that does this,
            // but for that we need peek/seek support and I'm not rewriting all of this project rn
            if input.version() >= 13.03 {
                input.skip(2)?;
            }

            return Ok(Tile {
                terrain,
                elevation,
                original_terrain: None,
            });
        }

        let (terrain, elevation, original_terrain) = if terrain == 0xFF {
            (input.read_u8()?, input.read_u8()?, Some(input.read_u8()?))
        } else {
            (terrain, input.read_u8()?, None)
        };
        Ok(Tile {
            terrain,
            elevation,
            original_terrain,
        })
    }
}

impl WritableHeaderElement for Tile {
    /// Write a tile to an output stream.
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        match self.original_terrain {
            Some(t) => {
                output.write_u8(0xFF)?;
                output.write_u8(self.terrain)?;
                output.write_u8(self.elevation)?;
                output.write_u8(t)?;
            }
            None => {
                output.write_u8(self.terrain)?;
                output.write_u8(self.elevation)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MapZone {
    /// Zone information—this is a Vec<> of a fixed size, and can only be accessed as a slice
    /// through the `info()` accessors to prevent modifications to the size.
    info: Vec<i8>,
    tiles: Vec<i32>,
    pub zone_map: Vec<i8>,
    pub passability_rules: Vec<f32>,
    pub num_zones: u32,
}

impl Default for MapZone {
    fn default() -> Self {
        Self {
            info: vec![0; 255],
            tiles: vec![0; 255],
            zone_map: Default::default(),
            passability_rules: Default::default(),
            num_zones: Default::default(),
        }
    }
}

impl MapZone {
    pub fn info(&self) -> &[i8] {
        assert_eq!(self.info.len(), 255);
        &self.info
    }

    pub fn info_mut(&mut self) -> &mut [i8] {
        assert_eq!(self.info.len(), 255);
        &mut self.info
    }

    pub fn tiles(&self) -> &[i32] {
        assert_eq!(self.tiles.len(), 255);
        &self.tiles
    }

    pub fn tiles_mut(&mut self) -> &mut [i32] {
        assert_eq!(self.tiles.len(), 255);
        &mut self.tiles
    }
}

impl ReadableHeaderElement for MapZone {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut zone = Self::default();
        input.read_i8_into(&mut zone.info)?;
        input.read_i32_into::<LE>(&mut zone.tiles)?;
        zone.zone_map = vec![0; input.tile_count()];
        input.read_i8_into(&mut zone.zone_map)?;

        // this changed in HD/DE, but I have no clue
        if input.version() > 11.93 {
            input.skip((2048 + (input.tile_count() * 2)) as u64)?
        } else {
            input.read_i8_into(&mut zone.info)?;
            input.read_i32_into::<LE>(&mut zone.tiles)?;
            zone.zone_map = vec![0; input.tile_count()];
            input.read_i8_into(&mut zone.zone_map)?;
        }
        let num_rules = input.read_u32::<LE>()?;
        zone.passability_rules = vec![0.0; num_rules as usize];
        input.read_f32_into::<LE>(&mut zone.passability_rules)?;
        zone.num_zones = input.read_u32::<LE>()?;
        Ok(zone)
    }
}

impl WritableHeaderElement for MapZone {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        for val in &self.info {
            output.write_i8(*val)?;
        }
        for val in &self.tiles {
            output.write_i32::<LE>(*val)?;
        }
        for val in &self.zone_map {
            output.write_i8(*val)?;
        }
        output.write_u32::<LE>(self.passability_rules.len().try_into().unwrap())?;
        for val in &self.passability_rules {
            output.write_f32::<LE>(*val)?;
        }
        output.write_u32::<LE>(self.num_zones)?;
        Ok(())
    }
}

///
#[derive(Debug, Default, Clone)]
pub struct VisibilityMap {
    /// Width of the visibility map.
    pub width: u32,
    /// Height of the visibility map.
    pub height: u32,
    /// Visibility flags for each tile.
    pub visibility: Vec<u32>,
}

impl ReadableHeaderElement for VisibilityMap {
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let width = input.read_u32::<LE>()?;
        let height = input.read_u32::<LE>()?;
        let mut visibility = vec![0; (width * height).try_into().unwrap()];
        input.read_u32_into::<LE>(&mut visibility)?;
        Ok(Self {
            width,
            height,
            visibility,
        })
    }
}

impl WritableHeaderElement for VisibilityMap {
    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u32::<LE>(self.width)?;
        output.write_u32::<LE>(self.height)?;
        for value in &self.visibility {
            output.write_u32::<LE>(*value)?;
        }
        Ok(())
    }
}

/// Information about the map being played.
#[derive(Debug, Default, Clone)]
pub struct Map {
    /// Width of the map.
    pub width: u32,
    /// Height of the map.
    pub height: u32,
    /// Map zones.
    pub zones: Vec<MapZone>,
    /// Is the "All Visible" flag set?
    pub all_visible: bool,
    /// Is fog of war enabled?
    pub fog_of_war: bool,
    /// The tiles in this map, containing terrain and elevation data.
    pub tiles: Vec<Tile>,
    /// The visibility map, containing line of sight data for each player.
    pub visibility: VisibilityMap,
}

impl ReadableHeaderElement for Map {
    /// Read map data from an input stream.
    fn read_from<R: Read>(input: &mut RecordingHeaderReader<R>) -> Result<Self> {
        let mut map = Self::default();
        map.width = input.read_u32::<LE>()?;
        map.height = input.read_u32::<LE>()?;
        input.set_map_size(map.width, map.height);
        let num_zones = input.read_u32::<LE>()?;
        dbg!(num_zones);
        map.zones = Vec::with_capacity(num_zones.try_into().unwrap());
        for _ in 0..num_zones {
            map.zones.push(MapZone::read_from(input)?);
        }
        map.all_visible = input.read_u8()? != 0;
        map.fog_of_war = input.read_u8()? != 0;
        map.tiles = Vec::with_capacity((map.width * map.height).try_into().unwrap());
        for _ in 0..(map.width * map.height) {
            map.tiles.push(Tile::read_from(input)?);
        }

        let _umv = {
            let data_count = input.read_u32::<LE>()?;
            let _capacity = input.read_u32::<LE>()?;
            input.skip(u64::from(data_count) * 4)?;
            for _ in 0..data_count {
                let count = input.read_u32::<LE>()?;
                input.skip(u64::from(count) * 8)?;
            }
        };

        map.visibility = VisibilityMap::read_from(input)?;

        Ok(map)
    }
}
