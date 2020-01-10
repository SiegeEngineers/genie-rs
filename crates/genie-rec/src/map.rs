use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::convert::TryInto;
use std::io::{Read, Write};

#[derive(Debug, Default, Clone)]
pub struct Tile {
    pub terrain: u8,
    pub elevation: u8,
    pub original_terrain: Option<u8>,
}

impl Tile {
    fn read_from(mut input: impl Read) -> Result<Self> {
        let terrain = input.read_u8()?;
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

    fn write_to(&self, mut output: impl Write) -> Result<()> {
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
    pub fn read_from(mut input: impl Read, map_size: (u32, u32)) -> Result<Self> {
        let mut zone = Self::default();
        for val in zone.info.iter_mut() {
            *val = input.read_i8()?;
        }
        for val in zone.tiles.iter_mut() {
            *val = input.read_i32::<LE>()?;
        }
        zone.zone_map = vec![0; (map_size.0 * map_size.1).try_into().unwrap()];
        for val in zone.zone_map.iter_mut() {
            *val = input.read_i8()?;
        }

        let num_rules = input.read_u32::<LE>()?;
        zone.passability_rules = vec![0.0; num_rules.try_into().unwrap()];
        for val in zone.passability_rules.iter_mut() {
            *val = input.read_f32::<LE>()?;
        }

        zone.num_zones = input.read_u32::<LE>()?;
        Ok(zone)
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
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

#[derive(Debug, Default, Clone)]
pub struct VisibilityMap {
    pub width: u32,
    pub height: u32,
    pub visibility: Vec<u32>,
}

impl VisibilityMap {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let width = input.read_u32::<LE>()?;
        let height = input.read_u32::<LE>()?;
        let mut visibility = vec![0; (width * height).try_into().unwrap()];
        for value in visibility.iter_mut() {
            *value = input.read_u32::<LE>()?;
        }
        Ok(Self {
            width,
            height,
            visibility,
        })
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_u32::<LE>(self.width)?;
        output.write_u32::<LE>(self.height)?;
        for value in &self.visibility {
            output.write_u32::<LE>(*value)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub zones: Vec<MapZone>,
    pub all_visible: bool,
    pub fog_of_war: bool,
    pub terrain: Vec<Tile>,
    pub visibility: VisibilityMap,
}

impl Map {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut map = Self::default();
        map.width = input.read_u32::<LE>()?;
        map.height = input.read_u32::<LE>()?;
        let num_zones = input.read_u32::<LE>()?;
        map.zones = Vec::with_capacity(num_zones.try_into().unwrap());
        for _ in 0..num_zones {
            map.zones
                .push(MapZone::read_from(&mut input, (map.width, map.height))?);
        }
        map.all_visible = input.read_u8()? != 0;
        map.fog_of_war = input.read_u8()? != 0;
        map.terrain = Vec::with_capacity((map.width * map.height).try_into().unwrap());
        for _ in 0..(map.width * map.height) {
            map.terrain.push(Tile::read_from(&mut input)?);
        }

        let _umv = {
            let data_count = input.read_u32::<LE>()?;
            let _capacity = input.read_u32::<LE>()?;
            skip(&mut input, u64::from(data_count) * 4)?;
            for _ in 0..data_count {
                let count = input.read_u32::<LE>()?;
                skip(&mut input, u64::from(count) * 8)?;
            }
            ()
        };

        map.visibility = VisibilityMap::read_from(&mut input)?;

        Ok(map)
    }

    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        unimplemented!()
    }
}

fn skip<R: Read>(input: &mut R, bytes: u64) -> std::io::Result<()> {
    std::io::copy(&mut input.by_ref().take(bytes), &mut std::io::sink())?;
    Ok(())
}
