use crate::Version;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Result, Write};

#[derive(Debug, Default, Clone)]
pub struct TerrainPassGraphic {
    exit_tile_id: i32,
    enter_tile_id: i32,
    walk_tile_id: i32,
    walk_rate: Option<f32>,
    replication_amount: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct TerrainRestriction {
    passability: Vec<f32>,
    pass_graphics: Vec<TerrainPassGraphic>,
}

#[derive(Debug, Default, Clone)]
pub struct TileSize {
    pub width: i16,
    pub height: i16,
    pub delta_z: i16,
}

#[derive(Debug, Default, Clone)]
pub struct TerrainAnimation {
    pub enabled: bool,
    num_frames: i16,
    num_pause_frames: i16,
    frame_interval: f32,
    replay_delay: f32,
    frame: i16,
    draw_frame: i16,
    animate_last: f32,
    frame_changed: bool,
    drawn: bool,
}

#[derive(Debug, Default, Clone)]
pub struct TerrainSpriteFrame {
    pub num_frames: i16,
    pub num_facets: i16,
    pub frame_id: i16,
}

#[derive(Debug, Default, Clone)]
pub struct TerrainObject {
    pub object_id: i16,
    pub density: i16,
    pub placement_flag: i8,
}

#[derive(Debug, Default, Clone)]
pub struct Terrain {
    pub enabled: bool,
    random: u8,
    name: String,
    slp_name: String,
    pub slp_id: i32,
    pub sound_id: i32,
    blend_priority: Option<i32>,
    blend_mode: Option<i32>,
    pub minimap_color_high: u8,
    pub minimap_color_medium: u8,
    pub minimap_color_low: u8,
    pub minimap_color_cliff_lt: u8,
    pub minimap_color_cliff_rt: u8,
    pub passable_terrain_id: Option<u8>,
    pub impassable_terrain_id: Option<u8>,
    pub animation: TerrainAnimation,
    pub elevation_sprites: Vec<TerrainSpriteFrame>,
    pub terrain_id_to_draw: Option<i16>,
    rows: i16,
    cols: i16,
    pub borders: Vec<i16>,
    pub terrain_objects: Vec<TerrainObject>,
}

#[derive(Debug, Default, Clone)]
pub struct TerrainBorder {
    pub enabled: bool,
    random: u8,
    name: String,
    slp_name: String,
    pub slp_id: i32,
    pub sound_id: i32,
    pub color: (u8, u8, u8),
    pub animation: TerrainAnimation,
    pub frames: Vec<Vec<TerrainSpriteFrame>>,
    /// Unused according to Chariot.
    draw_tile: i8,
    pub underlay_terrain: Option<i16>,
    pub border_style: i16,
}

impl TerrainPassGraphic {
    pub fn from<R: Read>(input: &mut R, version: Version) -> Result<Self> {
        let mut pass = TerrainPassGraphic::default();
        pass.exit_tile_id = input.read_i32::<LE>()?;
        pass.enter_tile_id = input.read_i32::<LE>()?;
        pass.walk_tile_id = input.read_i32::<LE>()?;
        if version.is_swgb() {
            pass.walk_rate = Some(input.read_f32::<LE>()?);
        } else {
            pass.replication_amount = Some(input.read_i32::<LE>()?);
        }
        Ok(pass)
    }
}

impl TerrainRestriction {
    pub fn from<R: Read>(input: &mut R, version: Version, num_terrains: u16) -> Result<Self> {
        let mut passability = vec![0.0; num_terrains as usize];
        for value in passability.iter_mut() {
            *value = input.read_f32::<LE>()?;
        }

        // Apparently AoK+ only
        let mut pass_graphics = Vec::with_capacity(num_terrains as usize);
        for _ in 0..num_terrains {
            pass_graphics.push(TerrainPassGraphic::from(input, version)?);
        }

        Ok(Self {
            passability,
            pass_graphics,
        })
    }
}

impl TileSize {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let width = input.read_i16::<LE>()?;
        let height = input.read_i16::<LE>()?;
        let delta_z = input.read_i16::<LE>()?;
        Ok(Self {
            width,
            height,
            delta_z,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i16::<LE>(self.width)?;
        output.write_i16::<LE>(self.height)?;
        output.write_i16::<LE>(self.delta_z)?;
        Ok(())
    }
}

impl TerrainAnimation {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut anim = TerrainAnimation::default();
        anim.enabled = input.read_u8()? != 0;
        anim.num_frames = input.read_i16::<LE>()?;
        anim.num_pause_frames = input.read_i16::<LE>()?;
        anim.frame_interval = input.read_f32::<LE>()?;
        anim.replay_delay = input.read_f32::<LE>()?;
        anim.frame = input.read_i16::<LE>()?;
        anim.draw_frame = input.read_i16::<LE>()?;
        anim.animate_last = input.read_f32::<LE>()?;
        anim.frame_changed = input.read_u8()? != 0;
        anim.drawn = input.read_u8()? != 0;
        Ok(anim)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(if self.enabled { 1 } else { 0 })?;
        output.write_i16::<LE>(self.num_frames)?;
        output.write_i16::<LE>(self.num_pause_frames)?;
        output.write_f32::<LE>(self.frame_interval)?;
        output.write_f32::<LE>(self.replay_delay)?;
        output.write_i16::<LE>(self.frame)?;
        output.write_i16::<LE>(self.draw_frame)?;
        output.write_f32::<LE>(self.animate_last)?;
        output.write_u8(if self.frame_changed { 1 } else { 0 })?;
        output.write_u8(if self.drawn { 1 } else { 0 })?;
        Ok(())
    }
}

impl TerrainSpriteFrame {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let num_frames = input.read_i16::<LE>()?;
        let num_facets = input.read_i16::<LE>()?;
        let frame_id = input.read_i16::<LE>()?;
        Ok(Self {
            num_frames,
            num_facets,
            frame_id,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i16::<LE>(self.num_frames)?;
        output.write_i16::<LE>(self.num_facets)?;
        output.write_i16::<LE>(self.frame_id)?;
        Ok(())
    }
}

impl Terrain {
    pub fn from<R: Read>(input: &mut R, _version: Version, num_terrains: u16) -> Result<Self> {
        let mut terrain = Terrain::default();
        terrain.enabled = input.read_u8()? != 0;
        terrain.random = input.read_u8()?;
        let mut name = [0; 13];
        input.read_exact(&mut name)?;
        terrain.name =
            String::from_utf8(name.iter().cloned().take_while(|b| *b != 0).collect()).unwrap();
        input.read_exact(&mut name)?;
        terrain.slp_name =
            String::from_utf8(name.iter().cloned().take_while(|b| *b != 0).collect()).unwrap();
        terrain.slp_id = input.read_i32::<LE>()?;
        let _slp_pointer = input.read_i32::<LE>()?;
        terrain.sound_id = input.read_i32::<LE>()?;
        terrain.blend_priority = Some(input.read_i32::<LE>()?);
        terrain.blend_mode = Some(input.read_i32::<LE>()?);
        terrain.minimap_color_high = input.read_u8()?;
        terrain.minimap_color_medium = input.read_u8()?;
        terrain.minimap_color_low = input.read_u8()?;
        terrain.minimap_color_cliff_lt = input.read_u8()?;
        terrain.minimap_color_cliff_rt = input.read_u8()?;
        terrain.passable_terrain_id = match input.read_u8()? {
            0xFF => None,
            id => Some(id),
        };
        terrain.impassable_terrain_id = match input.read_u8()? {
            0xFF => None,
            id => Some(id),
        };
        terrain.animation = TerrainAnimation::from(input)?;
        for _ in 0..19 {
            terrain
                .elevation_sprites
                .push(TerrainSpriteFrame::from(input)?);
        }
        terrain.terrain_id_to_draw = match input.read_i16::<LE>()? {
            -1 => None,
            id => Some(id),
        };
        terrain.rows = input.read_i16::<LE>()?;
        terrain.cols = input.read_i16::<LE>()?;
        for _ in 0..num_terrains {
            terrain.borders.push(input.read_i16::<LE>()?);
        }

        let mut terrain_objects = vec![TerrainObject::default(); 30];
        for object in terrain_objects.iter_mut() {
            object.object_id = input.read_i16::<LE>()?;
        }
        for object in terrain_objects.iter_mut() {
            object.density = input.read_i16::<LE>()?;
        }
        for object in terrain_objects.iter_mut() {
            object.placement_flag = input.read_i8()?;
        }

        let _num_terrain_objects = input.read_u16::<LE>()?;
        // Why is num_terrain_objects always 0?
        // terrain_objects.truncate(num_terrain_objects as usize);
        terrain.terrain_objects = terrain_objects;

        let _padding = input.read_u16::<LE>()?;

        Ok(terrain)
    }
}

impl TerrainBorder {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut border = TerrainBorder::default();
        border.enabled = input.read_u8()? != 0;
        border.random = input.read_u8()?;
        let mut name = [0; 13];
        input.read_exact(&mut name)?;
        border.name =
            String::from_utf8(name.iter().cloned().take_while(|b| *b != 0).collect()).unwrap();
        input.read_exact(&mut name)?;
        border.slp_name =
            String::from_utf8(name.iter().cloned().take_while(|b| *b != 0).collect()).unwrap();
        border.slp_id = input.read_i32::<LE>()?;
        let _slp_pointer = input.read_i32::<LE>()?;
        border.sound_id = input.read_i32::<LE>()?;
        border.color = (input.read_u8()?, input.read_u8()?, input.read_u8()?);
        border.animation = TerrainAnimation::from(input)?;

        for _ in 0..19 {
            let mut frames_list = vec![TerrainSpriteFrame::default(); 12];
            for frame in frames_list.iter_mut() {
                *frame = TerrainSpriteFrame::from(input)?;
            }
            border.frames.push(frames_list);
        }

        border.draw_tile = input.read_i8()?;
        // Padding
        input.read_u8()?;
        border.underlay_terrain = match input.read_i16::<LE>()? {
            -1 => None,
            id => Some(id),
        };
        border.border_style = input.read_i16::<LE>()?;

        Ok(border)
    }
}
