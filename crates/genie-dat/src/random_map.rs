//! Types related to the random map data from Age of Empires 1.

#![allow(missing_docs)]

use crate::unit_type::UnitTypeID;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::convert::TryInto;
use std::io::{Read, Result, Write};

#[derive(Debug, Default, Clone)]
pub struct RandomMapInfo {
    id: i32,
    pub borders: (i32, i32, i32, i32),
    pub border_fade: i32,
    pub water_border: i32,
    pub base_terrain: i32,
    pub land_percent: i32,
    pub lands: Vec<RandomMapLand>,
    pub terrains: Vec<RandomMapTerrain>,
    pub objects: Vec<RandomMapObject>,
    pub elevations: Vec<RandomMapElevation>,
}

impl RandomMapInfo {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut info = Self::default();
        info.id = input.read_i32::<LE>()?;
        info.borders = (
            input.read_i32::<LE>()?,
            input.read_i32::<LE>()?,
            input.read_i32::<LE>()?,
            input.read_i32::<LE>()?,
        );
        info.border_fade = input.read_i32::<LE>()?;
        info.water_border = input.read_i32::<LE>()?;
        info.base_terrain = input.read_i32::<LE>()?;
        info.land_percent = input.read_i32::<LE>()?;

        let _some_id = input.read_i32::<LE>()?;
        let num_lands = input.read_u32::<LE>()?;
        let _pointer = input.read_u32::<LE>()?;
        let num_terrains = input.read_u32::<LE>()?;
        let _pointer = input.read_u32::<LE>()?;
        let num_objects = input.read_u32::<LE>()?;
        let _pointer = input.read_u32::<LE>()?;
        let num_elevations = input.read_u32::<LE>()?;
        let _pointer = input.read_u32::<LE>()?;

        info.lands = vec![RandomMapLand::default(); num_lands.try_into().unwrap()];
        info.terrains = vec![RandomMapTerrain::default(); num_terrains.try_into().unwrap()];
        info.objects = vec![RandomMapObject::default(); num_objects.try_into().unwrap()];
        info.elevations = vec![RandomMapElevation::default(); num_elevations.try_into().unwrap()];

        Ok(info)
    }

    pub fn finish<R: Read>(&mut self, input: &mut R) -> Result<()> {
        // duplicate data
        std::io::copy(&mut input.by_ref().take(44), &mut std::io::sink())?;
        for land in self.lands.iter_mut() {
            *land = RandomMapLand::read_from(input)?;
        }

        // duplicate data
        std::io::copy(&mut input.by_ref().take(8), &mut std::io::sink())?;
        for terrain in self.terrains.iter_mut() {
            *terrain = RandomMapTerrain::read_from(input)?;
        }

        // duplicate data
        std::io::copy(&mut input.by_ref().take(8), &mut std::io::sink())?;
        for object in self.objects.iter_mut() {
            *object = RandomMapObject::read_from(input)?;
        }

        // duplicate data
        std::io::copy(&mut input.by_ref().take(8), &mut std::io::sink())?;
        for elevation in self.elevations.iter_mut() {
            *elevation = RandomMapElevation::read_from(input)?;
        }
        Ok(())
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.id)?;
        output.write_i32::<LE>(self.borders.0)?;
        output.write_i32::<LE>(self.borders.1)?;
        output.write_i32::<LE>(self.borders.2)?;
        output.write_i32::<LE>(self.borders.3)?;
        output.write_i32::<LE>(self.border_fade)?;
        output.write_i32::<LE>(self.water_border)?;
        output.write_i32::<LE>(self.base_terrain)?;
        output.write_i32::<LE>(self.land_percent)?;

        output.write_i32::<LE>(0)?; // some id
        output.write_u32::<LE>(self.lands.len().try_into().unwrap())?;
        output.write_u32::<LE>(0)?; // pointer
        output.write_u32::<LE>(self.terrains.len().try_into().unwrap())?;
        output.write_u32::<LE>(0)?; // pointer
        output.write_u32::<LE>(self.objects.len().try_into().unwrap())?;
        output.write_u32::<LE>(0)?; // pointer
        output.write_u32::<LE>(self.elevations.len().try_into().unwrap())?;
        output.write_u32::<LE>(0)?; // pointer

        Ok(())
    }

    pub fn write_commands_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.borders.0)?;
        output.write_i32::<LE>(self.borders.1)?;
        output.write_i32::<LE>(self.borders.2)?;
        output.write_i32::<LE>(self.borders.3)?;
        output.write_i32::<LE>(self.border_fade)?;
        output.write_i32::<LE>(self.water_border)?;
        output.write_i32::<LE>(self.base_terrain)?;
        output.write_i32::<LE>(self.land_percent)?;
        output.write_u32::<LE>(0)?; // some id

        output.write_u32::<LE>(self.lands.len().try_into().unwrap())?;
        output.write_u32::<LE>(0)?; // pointer
        for land in &self.lands {
            land.write_to(output)?;
        }
        output.write_u32::<LE>(self.terrains.len().try_into().unwrap())?;
        output.write_u32::<LE>(0)?; // pointer
        for terrain in &self.terrains {
            terrain.write_to(output)?;
        }
        output.write_u32::<LE>(self.objects.len().try_into().unwrap())?;
        output.write_u32::<LE>(0)?; // pointer
        for object in &self.objects {
            object.write_to(output)?;
        }
        output.write_u32::<LE>(self.elevations.len().try_into().unwrap())?;
        output.write_u32::<LE>(0)?; // pointer
        for elevation in &self.elevations {
            elevation.write_to(output)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct RandomMapLand {
    pub id: i32,
    pub terrain_type: u8,
    pub land_avoidance_tiles: i32,
    pub base_square_radius: i32,
    pub zone: i8,
    pub placement_type: i8,
    pub x: i32,
    pub y: i32,
    pub amount_of_land_used_percent: i8,
    pub by_player_flag: i8,
    pub radius: i32,
    pub fade: i32,
    pub clumpiness_factor: i32,
}

impl RandomMapLand {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut land = Self::default();
        land.id = input.read_i32::<LE>()?;
        land.terrain_type = input.read_u8()?;
        let _padding = input.read_u16::<LE>()?;
        let _padding = input.read_u8()?;
        land.land_avoidance_tiles = input.read_i32::<LE>()?;
        land.base_square_radius = input.read_i32::<LE>()?;
        land.zone = input.read_i8()?;
        land.placement_type = input.read_i8()?;
        let _padding = input.read_u16::<LE>()?;
        land.x = input.read_i32::<LE>()?;
        land.y = input.read_i32::<LE>()?;
        land.amount_of_land_used_percent = input.read_i8()?;
        land.by_player_flag = input.read_i8()?;
        let _padding = input.read_u16::<LE>()?;
        land.radius = input.read_i32::<LE>()?;
        land.fade = input.read_i32::<LE>()?;
        land.clumpiness_factor = input.read_i32::<LE>()?;
        Ok(land)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.id)?;
        output.write_u8(self.terrain_type)?;
        output.write_u16::<LE>(0)?;
        output.write_u8(0)?;
        output.write_i32::<LE>(self.land_avoidance_tiles)?;
        output.write_i32::<LE>(self.base_square_radius)?;
        output.write_i8(self.zone)?;
        output.write_i8(self.placement_type)?;
        output.write_u16::<LE>(0)?;
        output.write_i32::<LE>(self.x)?;
        output.write_i32::<LE>(self.y)?;
        output.write_i8(self.amount_of_land_used_percent)?;
        output.write_i8(self.by_player_flag)?;
        output.write_u16::<LE>(0)?;
        output.write_i32::<LE>(self.radius)?;
        output.write_i32::<LE>(self.fade)?;
        output.write_i32::<LE>(self.clumpiness_factor)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct RandomMapTerrain {
    pub percent: i32,
    pub terrain_type: i32,
    pub clumps: i32,
    pub spacing: i32,
    pub base_terrain_type: i32,
    pub clumpiness_factor: i32,
}

impl RandomMapTerrain {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut terrain = Self::default();
        terrain.percent = input.read_i32::<LE>()?;
        terrain.terrain_type = input.read_i32::<LE>()?;
        terrain.clumps = input.read_i32::<LE>()?;
        terrain.spacing = input.read_i32::<LE>()?;
        terrain.base_terrain_type = input.read_i32::<LE>()?;
        terrain.clumpiness_factor = input.read_i32::<LE>()?;
        Ok(terrain)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.percent)?;
        output.write_i32::<LE>(self.terrain_type)?;
        output.write_i32::<LE>(self.clumps)?;
        output.write_i32::<LE>(self.spacing)?;
        output.write_i32::<LE>(self.base_terrain_type)?;
        output.write_i32::<LE>(self.clumpiness_factor)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct RandomMapObject {
    pub unit_type: UnitTypeID,
    pub terrain_type: i32,
    pub group_flag: i8,
    pub scale_flag: i8,
    pub group_size: i32,
    pub group_size_variance: i32,
    pub group_count: i32,
    pub group_area: i32,
    pub player_id: i32,
    pub land_id: i32,
    pub min_distance_to_players: i32,
    pub max_distance_to_players: i32,
}

impl RandomMapObject {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut object = Self::default();
        object.unit_type = input.read_u32::<LE>()?.try_into().unwrap();
        object.terrain_type = input.read_i32::<LE>()?;
        object.group_flag = input.read_i8()?;
        object.scale_flag = input.read_i8()?;
        let _padding = input.read_u16::<LE>()?;
        object.group_size = input.read_i32::<LE>()?;
        object.group_size_variance = input.read_i32::<LE>()?;
        object.group_count = input.read_i32::<LE>()?;
        object.group_area = input.read_i32::<LE>()?;
        object.player_id = input.read_i32::<LE>()?;
        object.land_id = input.read_i32::<LE>()?;
        object.min_distance_to_players = input.read_i32::<LE>()?;
        object.max_distance_to_players = input.read_i32::<LE>()?;
        Ok(object)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u32::<LE>(self.unit_type.try_into().unwrap())?;
        output.write_i32::<LE>(self.terrain_type)?;
        output.write_i8(self.group_flag)?;
        output.write_i8(self.scale_flag)?;
        output.write_u16::<LE>(0)?;
        output.write_i32::<LE>(self.group_size)?;
        output.write_i32::<LE>(self.group_size_variance)?;
        output.write_i32::<LE>(self.group_count)?;
        output.write_i32::<LE>(self.group_area)?;
        output.write_i32::<LE>(self.player_id)?;
        output.write_i32::<LE>(self.land_id)?;
        output.write_i32::<LE>(self.min_distance_to_players)?;
        output.write_i32::<LE>(self.max_distance_to_players)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct RandomMapElevation {
    pub percent: i32,
    pub height: i32,
    pub clumps: i32,
    pub spacing: i32,
    pub base_terrain_type: i32,
    pub base_elevation: i32,
}

impl RandomMapElevation {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let mut elevation = Self::default();
        elevation.percent = input.read_u32::<LE>()?.try_into().unwrap();
        elevation.height = input.read_i32::<LE>()?;
        elevation.clumps = input.read_i32::<LE>()?;
        elevation.spacing = input.read_i32::<LE>()?;
        elevation.base_terrain_type = input.read_i32::<LE>()?;
        elevation.base_elevation = input.read_i32::<LE>()?;
        Ok(elevation)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.percent)?;
        output.write_i32::<LE>(self.height)?;
        output.write_i32::<LE>(self.clumps)?;
        output.write_i32::<LE>(self.spacing)?;
        output.write_i32::<LE>(self.base_terrain_type)?;
        output.write_i32::<LE>(self.base_elevation)?;
        Ok(())
    }
}
