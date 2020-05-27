use crate::sound::SoundID;
use crate::sprite::{GraphicID, SpriteID};
use crate::unit_type::UnitTypeID;
use crate::FileVersion;
use arrayvec::ArrayString;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{
    fallible_try_from, fallible_try_into, infallible_try_into, read_opt_u16, read_opt_u32, MapInto,
};
use std::convert::TryInto;
use std::io::{Read, Result, Write};

/// An ID identifying a terrain.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TerrainID(u16);

impl From<u8> for TerrainID {
    fn from(n: u8) -> Self {
        TerrainID(n.into())
    }
}

impl From<u16> for TerrainID {
    fn from(n: u16) -> Self {
        TerrainID(n)
    }
}

impl From<TerrainID> for u16 {
    fn from(n: TerrainID) -> Self {
        n.0
    }
}

impl From<TerrainID> for usize {
    fn from(n: TerrainID) -> Self {
        n.0.into()
    }
}

fallible_try_into!(TerrainID, i16);
infallible_try_into!(TerrainID, u32);
fallible_try_from!(TerrainID, i32);
fallible_try_from!(TerrainID, u32);

type TerrainName = ArrayString<[u8; 13]>;

#[derive(Debug, Default, Clone)]
pub struct TerrainPassGraphic {
    exit_tile_sprite: Option<SpriteID>,
    enter_tile_sprite: Option<SpriteID>,
    walk_tile_sprite: Option<SpriteID>,
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
    pub object_id: UnitTypeID,
    pub density: i16,
    pub placement_flag: i8,
}

#[derive(Debug, Default, Clone)]
pub struct Terrain {
    /// Is this terrain enabled?
    pub enabled: bool,
    random: u8,
    /// Internal name of the terrain.
    name: TerrainName,
    /// Internal name of the SLP graphic.
    slp_name: TerrainName,
    /// SLP graphic ID for this terrain.
    pub slp_id: Option<GraphicID>,
    /// The Sound ID for this terrain.
    pub sound_id: Option<SoundID>,
    wwise_sound_id: Option<u32>,
    wwise_stop_sound_id: Option<u32>,
    blend_priority: Option<i32>,
    blend_mode: Option<i32>,
    /// The colour tiles with this terrain will have on the minimap when on a downhill slope.
    pub minimap_color_high: u8,
    /// The colour tiles with this terrain will have on the minimap when on a flat tile.
    pub minimap_color_medium: u8,
    /// The colour tiles with this terrain will have on the minimap when on an uphill slope.
    pub minimap_color_low: u8,
    /// The colour tiles with this terrain will have on the minimap when next to a cliff.
    pub minimap_color_cliff_lt: u8,
    /// The colour tiles with this terrain will have on the minimap when next to a cliff.
    pub minimap_color_cliff_rt: u8,
    pub passable_terrain_id: Option<u8>,
    pub impassable_terrain_id: Option<u8>,
    pub animation: TerrainAnimation,
    pub elevation_sprites: Vec<TerrainSpriteFrame>,
    pub terrain_id_to_draw: Option<TerrainID>,
    rows: i16,
    cols: i16,
    pub borders: Vec<i16>,
    pub terrain_objects: Vec<TerrainObject>,
}

#[derive(Debug, Default, Clone)]
pub struct TerrainBorder {
    pub enabled: bool,
    random: u8,
    name: TerrainName,
    slp_name: TerrainName,
    pub slp_id: Option<GraphicID>,
    pub sound_id: Option<SoundID>,
    pub color: (u8, u8, u8),
    pub animation: TerrainAnimation,
    pub frames: Vec<Vec<TerrainSpriteFrame>>,
    /// Unused according to Chariot.
    draw_tile: i8,
    pub underlay_terrain: Option<i16>,
    pub border_style: i16,
}

impl TerrainPassGraphic {
    pub fn read_from(mut input: impl Read, version: FileVersion) -> Result<Self> {
        let mut pass = TerrainPassGraphic::default();
        pass.exit_tile_sprite = read_opt_u32(&mut input)?;
        pass.enter_tile_sprite = read_opt_u32(&mut input)?;
        pass.walk_tile_sprite = read_opt_u32(&mut input)?;
        if version.is_swgb() {
            pass.walk_rate = Some(input.read_f32::<LE>()?);
        } else {
            pass.replication_amount = Some(input.read_i32::<LE>()?);
        }
        Ok(pass)
    }

    /// Serialize this object to a binary output stream.
    pub fn write_to(&self, mut output: impl Write, version: FileVersion) -> Result<()> {
        output.write_i32::<LE>(self.exit_tile_sprite.map_into().unwrap_or(-1))?;
        output.write_i32::<LE>(self.enter_tile_sprite.map_into().unwrap_or(-1))?;
        output.write_i32::<LE>(self.walk_tile_sprite.map_into().unwrap_or(-1))?;
        // TODO decide on correct default values for these
        if version.is_swgb() {
            output.write_f32::<LE>(self.walk_rate.unwrap_or(0.0))?;
        } else {
            output.write_i32::<LE>(self.replication_amount.unwrap_or(-1))?;
        }
        Ok(())
    }
}

impl TerrainRestriction {
    pub fn read_from(
        mut input: impl Read,
        version: FileVersion,
        num_terrains: u16,
    ) -> Result<Self> {
        let mut passability = vec![0.0; num_terrains as usize];
        for value in passability.iter_mut() {
            *value = input.read_f32::<LE>()?;
        }

        // Apparently AoK+ only
        let mut pass_graphics = Vec::with_capacity(num_terrains as usize);
        for _ in 0..num_terrains {
            pass_graphics.push(TerrainPassGraphic::read_from(&mut input, version)?);
        }

        Ok(Self {
            passability,
            pass_graphics,
        })
    }

    /// Serialize this object to a binary output stream.
    pub fn write_to(
        &self,
        mut output: impl Write,
        version: FileVersion,
        num_terrains: u16,
    ) -> Result<()> {
        assert_eq!(self.passability.len(), num_terrains.into());
        assert_eq!(self.pass_graphics.len(), num_terrains.into());
        for value in &self.passability {
            output.write_f32::<LE>(*value)?;
        }
        for graphic in &self.pass_graphics {
            graphic.write_to(&mut output, version)?;
        }
        Ok(())
    }
}

impl TileSize {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let width = input.read_i16::<LE>()?;
        let height = input.read_i16::<LE>()?;
        let delta_z = input.read_i16::<LE>()?;
        Ok(Self {
            width,
            height,
            delta_z,
        })
    }

    /// Serialize this object to a binary output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i16::<LE>(self.width)?;
        output.write_i16::<LE>(self.height)?;
        output.write_i16::<LE>(self.delta_z)?;
        Ok(())
    }
}

impl TerrainAnimation {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
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

    /// Serialize this object to a binary output stream.
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
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let num_frames = input.read_i16::<LE>()?;
        let num_facets = input.read_i16::<LE>()?;
        let frame_id = input.read_i16::<LE>()?;
        Ok(Self {
            num_frames,
            num_facets,
            frame_id,
        })
    }

    /// Serialize this object to a binary output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i16::<LE>(self.num_frames)?;
        output.write_i16::<LE>(self.num_facets)?;
        output.write_i16::<LE>(self.frame_id)?;
        Ok(())
    }
}

impl Terrain {
    /// Get the internal name of this terrain.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Read a Terrain object from an input stream.
    pub fn read_from(
        mut input: impl Read,
        version: FileVersion,
        num_terrains: u16,
    ) -> Result<Self> {
        let mut terrain = Terrain::default();
        terrain.enabled = input.read_u8()? != 0;
        terrain.random = input.read_u8()?;
        read_terrain_name(&mut input, &mut terrain.name)?;
        read_terrain_name(&mut input, &mut terrain.slp_name)?;
        // println!("{}", terrain.name);
        terrain.slp_id = read_opt_u32(&mut input)?;
        let _slp_pointer = input.read_i32::<LE>()?;
        terrain.sound_id = read_opt_u32(&mut input)?;
        if version.is_de2() {
            terrain.wwise_sound_id = read_opt_u32(&mut input)?;
            terrain.wwise_stop_sound_id = read_opt_u32(&mut input)?;
        } else {
            terrain.blend_priority = Some(input.read_i32::<LE>()?);
            terrain.blend_mode = Some(input.read_i32::<LE>()?);
        }
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
        terrain.animation = TerrainAnimation::read_from(&mut input)?;
        for _ in 0..19 {
            terrain
                .elevation_sprites
                .push(TerrainSpriteFrame::read_from(&mut input)?);
        }
        terrain.terrain_id_to_draw = read_opt_u16(&mut input)?;
        terrain.rows = input.read_i16::<LE>()?;
        terrain.cols = input.read_i16::<LE>()?;
        for _ in 0..num_terrains {
            terrain.borders.push(input.read_i16::<LE>()?);
        }

        let mut terrain_objects = vec![TerrainObject::default(); 30];
        for object in terrain_objects.iter_mut() {
            object.object_id = input.read_u16::<LE>()?.into();
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

    /// Serialize this object to a binary output stream.
    pub fn write_to<W: Write>(
        &self,
        output: &mut W,
        _version: FileVersion,
        num_terrains: u16,
    ) -> Result<()> {
        assert_eq!(self.borders.len(), num_terrains as usize);
        output.write_u8(if self.enabled { 1 } else { 0 })?;
        output.write_u8(self.random)?;
        write_terrain_name(output, &self.name)?;
        write_terrain_name(output, &self.slp_name)?;
        output.write_i32::<LE>(self.slp_id.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_i32::<LE>(0)?; // slp pointer
        output.write_i32::<LE>(self.sound_id.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_i32::<LE>(self.blend_priority.unwrap_or(-1))?;
        output.write_i32::<LE>(self.blend_mode.unwrap_or(-1))?;
        output.write_u8(self.minimap_color_high)?;
        output.write_u8(self.minimap_color_medium)?;
        output.write_u8(self.minimap_color_low)?;
        output.write_u8(self.minimap_color_cliff_lt)?;
        output.write_u8(self.minimap_color_cliff_rt)?;
        output.write_u8(self.passable_terrain_id.unwrap_or(0xFF))?;
        output.write_u8(self.impassable_terrain_id.unwrap_or(0xFF))?;
        self.animation.write_to(output)?;
        for frame in &self.elevation_sprites {
            frame.write_to(output)?;
        }
        output.write_i16::<LE>(
            self.terrain_id_to_draw
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(self.rows)?;
        output.write_i16::<LE>(self.cols)?;
        for border in &self.borders {
            output.write_i16::<LE>(*border)?;
        }

        for index in 0..30 {
            if let Some(object) = self.terrain_objects.get(index) {
                output.write_u16::<LE>(object.object_id.into())?;
            } else {
                output.write_u16::<LE>(0)?;
            }
        }
        for index in 0..30 {
            if let Some(object) = self.terrain_objects.get(index) {
                output.write_i16::<LE>(object.density)?;
            } else {
                output.write_i16::<LE>(0)?;
            }
        }
        for index in 0..30 {
            if let Some(object) = self.terrain_objects.get(index) {
                output.write_i8(object.placement_flag)?;
            } else {
                output.write_i8(0)?;
            }
        }
        output.write_u16::<LE>(self.terrain_objects.len() as u16)?;

        output.write_u16::<LE>(0)?; // padding

        Ok(())
    }
}

impl TerrainBorder {
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut border = TerrainBorder::default();
        border.enabled = input.read_u8()? != 0;
        border.random = input.read_u8()?;
        read_terrain_name(&mut input, &mut border.name)?;
        read_terrain_name(&mut input, &mut border.slp_name)?;
        border.slp_id = read_opt_u32(&mut input)?;
        let _slp_pointer = input.read_i32::<LE>()?;
        border.sound_id = read_opt_u32(&mut input)?;
        border.color = (input.read_u8()?, input.read_u8()?, input.read_u8()?);
        border.animation = TerrainAnimation::read_from(&mut input)?;
        for _ in 0..19 {
            let mut frames_list = vec![TerrainSpriteFrame::default(); 12];
            for frame in frames_list.iter_mut() {
                *frame = TerrainSpriteFrame::read_from(&mut input)?;
            }
            border.frames.push(frames_list);
        }

        border.draw_tile = input.read_i8()?;
        // Padding
        input.read_u8()?;
        border.underlay_terrain = read_opt_u16(&mut input)?;
        border.border_style = input.read_i16::<LE>()?;

        Ok(border)
    }

    /// Serialize this object to a binary output stream.
    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8(if self.enabled { 1 } else { 0 })?;
        output.write_u8(self.random)?;
        write_terrain_name(output, &self.name)?;
        write_terrain_name(output, &self.slp_name)?;
        output.write_i32::<LE>(self.slp_id.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_i32::<LE>(0)?; // slp pointer
        output.write_i32::<LE>(self.sound_id.map(|id| id.try_into().unwrap()).unwrap_or(-1))?;
        output.write_u8(self.color.0)?;
        output.write_u8(self.color.1)?;
        output.write_u8(self.color.2)?;
        self.animation.write_to(output)?;
        for frames_list in &self.frames {
            for frame in frames_list {
                frame.write_to(output)?;
            }
        }
        output.write_i8(self.draw_tile)?;
        output.write_u8(0)?; // padding
        output.write_i16::<LE>(
            self.underlay_terrain
                .map(|id| id.try_into().unwrap())
                .unwrap_or(-1),
        )?;
        output.write_i16::<LE>(self.border_style)?;
        Ok(())
    }
}

fn read_terrain_name<R: Read>(input: &mut R, output: &mut TerrainName) -> Result<()> {
    let bytes = &mut [0; 13];
    input.read_exact(bytes)?;
    bytes
        .iter()
        .cloned()
        .take_while(|b| *b != 0)
        .map(char::from)
        .for_each(|c| output.push(c));
    Ok(())
}

fn write_terrain_name<W: Write>(output: &mut W, name: &TerrainName) -> Result<()> {
    let bytes = &mut [0; 13];
    (&mut bytes[..name.len()]).copy_from_slice(name.as_bytes());
    output.write_all(bytes)?;
    Ok(())
}
