//! This module contains the data format reading/writing.
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::cmp::Ordering;
use std::io::{
    Read,
    Write,
    Result,
    Error,
    ErrorKind,
};
use crate::util::*;
use crate::types::*;
use crate::triggers::TriggerSystem;
use crate::VersionBundle;

/// Compare floats with some error.
macro_rules! cmp_float {
    ($id:ident == $val:expr) => {
        ($id - $val).abs() < std::f32::EPSILON
    };
    ($id:ident != $val:expr) => {
        ($id - $val).abs() > std::f32::EPSILON
    };
}

fn cmp_scx_version(a: SCXVersion, b: SCXVersion) -> Ordering {
    match a[0].cmp(&b[0]) {
        Ordering::Equal => {},
        ord => return ord,
    }
    match a[2].cmp(&b[2]) {
        Ordering::Equal => {},
        ord => return ord,
    }
    a[3].cmp(&b[3])
}

// pub enum LostInformation {
//     DisabledTechs(i32, i32),
//     DisabledUnits(i32, i32),
//     DisabledBuildings(i32, i32),
//     MapType,
// }

#[derive(Debug)]
pub struct DLCOptions {
     pub version: i32,
     pub game_data_set: DataSet,
     pub dependencies: Vec<DLCPackage>,
}

impl Default for DLCOptions {
    fn default() -> Self {
        Self {
            version: 1000,
            game_data_set: DataSet::BaseGame,
            dependencies: vec![
                DLCPackage::AgeOfKings,
                DLCPackage::AgeOfConquerors,
            ],
        }
    }
}

impl DLCOptions {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
         // If version is 0 or 1, it's actually the dataset identifier from
         // before DLCOptions was versioned.
         let version_or_data_set = input.read_i32::<LE>()?;
         let game_data_set = DataSet::try_from(
             if version_or_data_set == 0 || version_or_data_set == 1 {
                 version_or_data_set
             } else {
                 input.read_i32::<LE>()?
             })?;

         // Set version to 0 for old DLCOptions.
         let version = if version_or_data_set == 1 {
             0
         } else {
             version_or_data_set
         };

         let num_dependencies = input.read_u32::<LE>()?;
         let mut dependencies = vec![];
         for _ in 0..num_dependencies {
             dependencies.push(DLCPackage::try_from(
                     input.read_i32::<LE>()?)?);
         }

         Ok(DLCOptions {
             version,
             game_data_set,
             dependencies
         })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u32::<LE>(1000)?;
        output.write_i32::<LE>(self.game_data_set.into())?;
        output.write_u32::<LE>(self.dependencies.len() as u32)?;
        for dlc_id in &self.dependencies {
            output.write_i32::<LE>((*dlc_id).into())?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SCXHeader {
    /// Version of the header.
    ///
    /// Versions 2 and up include a save timestamp.
    /// Versions 3 and up contain HD Edition DLC information.
    pub version: u32,
    /// Unix timestamp when this scenario was created, in seconds.
    pub timestamp: u32,
    /// Description text about the scenario.
    pub description: Option<String>,
    /// Whether the scenario has any victory conditions for singleplayer.
    pub any_sp_victory: bool,
    /// How many players are supported by this scenario.
    pub active_player_count: u32,
    /// HD Edition DLC information.
    pub dlc_options: Option<DLCOptions>,
}

impl SCXHeader {
    /// Parse an SCX header from a byte stream.
    pub fn from<R: Read>(input: &mut R, format_version: SCXVersion) -> Result<SCXHeader> {
        let _header_size = input.read_u32::<LE>()?;
        let version = input.read_u32::<LE>()?;
        let timestamp = if version >= 2 {
            input.read_u32::<LE>()?
        } else {
            0
        };
        let description_length = if format_version == *b"3.13" {
            // Skip unknown value
            input.read_u16::<LE>()?;
            input.read_u16::<LE>()? as usize
        } else {
            input.read_u32::<LE>()? as usize
        };
        let description = read_str(input, description_length)?;

        let any_sp_victory = input.read_u32::<LE>()? != 0;
        let active_player_count = input.read_u32::<LE>()?;

        let dlc_options = if version > 2 && format_version != *b"3.13" {
            Some(DLCOptions::from(input)?)
        } else {
            None
        };

        Ok(SCXHeader {
            version,
            timestamp,
            description,
            any_sp_victory,
            active_player_count,
            dlc_options,
        })
    }

    /// Serialize an SCX header to a byte stream.
    pub fn write_to<W: Write>(&self, output: &mut W, format_version: SCXVersion, version: u32) -> Result<()> {
        let mut intermediate = vec![];

        intermediate.write_u32::<LE>(version)?;

        if version >= 2 {
            let system_time = std::time::SystemTime::now();
            let duration = system_time.duration_since(std::time::UNIX_EPOCH);
            intermediate.write_u32::<LE>(duration.map(|d| d.as_secs() as u32).unwrap_or(0))?;
        }

        let mut description_bytes = vec![];
        if let Some(ref description) = self.description {
            description_bytes.write_all(description.as_bytes())?;
            description_bytes.push(0);
        }
        if format_version == *b"3.13" {
            assert!(description_bytes.len() <= std::u16::MAX as usize, "description length must fit in u16");
            intermediate.write_u16::<LE>(description_bytes.len() as u16)?;
        } else {
            assert!(description_bytes.len() <= std::u32::MAX as usize, "description length must fit in u32");
            intermediate.write_u32::<LE>(description_bytes.len() as u32)?;
        }
        intermediate.write_all(&description_bytes)?;

        intermediate.write_u32::<LE>(if self.any_sp_victory { 1 } else { 0 })?;
        intermediate.write_u32::<LE>(self.active_player_count)?;

        if version > 2 && format_version != *b"3.13" {
            let def = DLCOptions::default();
            let dlc_options = match self.dlc_options {
                Some(ref options) => options,
                None => &def,
            };
            dlc_options.write_to(&mut intermediate)?;
        }

        output.write_u32::<LE>(intermediate.len() as u32)?;
        output.write_all(&intermediate)?;

        Ok(())
    }
}

#[derive(Debug)]
struct PlayerBaseProperties {
    pub(crate) posture: i32,
    pub(crate) player_type: i32,
    pub(crate) civilization: i32,
    pub(crate) active: i32,
}

#[derive(Debug)]
struct PlayerFiles {
    /// Obsolete.
    pub(crate) build_list: Option<String>,
    /// Obsolete.
    pub(crate) city_plan: Option<String>,
    /// String content of the AI of this player.
    pub(crate) ai_rules: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct BitmapColor(pub u8, pub u8, pub u8, pub u8);

impl BitmapColor {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let r = input.read_u8()?;
        let g = input.read_u8()?;
        let b = input.read_u8()?;
        let reserved = input.read_u8()?;
        Ok(BitmapColor(r, g, b, reserved))
    }
}

#[derive(Debug)]
struct BitmapInfo {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    xpels_per_meter: i32,
    ypels_per_meter: i32,
    clr_used: u32,
    clr_important: u32,
    colors: Vec<BitmapColor>,
}

impl BitmapInfo {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let size = input.read_u32::<LE>()?;
        let width = input.read_i32::<LE>()?;
        let height = input.read_i32::<LE>()?;
        let planes = input.read_u16::<LE>()?;
        let bit_count = input.read_u16::<LE>()?;
        let compression = input.read_u32::<LE>()?;
        let size_image = input.read_u32::<LE>()?;
        let xpels_per_meter = input.read_i32::<LE>()?;
        let ypels_per_meter = input.read_i32::<LE>()?;
        let clr_used = input.read_u32::<LE>()?;
        let clr_important = input.read_u32::<LE>()?;
        let mut colors = Vec::with_capacity(256);

        for _ in 0..256 {
            colors.push(BitmapColor::from(input)?);
        }

        Ok(BitmapInfo {
            size,
            width,
            height,
            planes,
            bit_count,
            compression,
            size_image,
            xpels_per_meter,
            ypels_per_meter,
            clr_used,
            clr_important,
            colors,
        })
    }
}

#[derive(Debug)]
struct Bitmap {
    own_memory: u32,
    width: u32,
    height: u32,
    orientation: u16,
    info: BitmapInfo,
    pixels: Vec<u8>,
}

impl Bitmap {
    pub fn from<R: Read>(input: &mut R) -> Result<Option<Self>> {
        let own_memory = input.read_u32::<LE>()?;
        let width = input.read_u32::<LE>()?;
        let height = input.read_u32::<LE>()?;
        let orientation = input.read_u16::<LE>()?;

        if width > 0 && height > 0 {
            let info = BitmapInfo::from(input)?;
            let mut pixels = vec![0u8; (height * ((width + 3) & !3)) as usize];
            input.read_exact(&mut pixels)?;
            Ok(Some(Bitmap {
                own_memory,
                width,
                height,
                orientation,
                info,
                pixels,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub(crate) struct RGEScen {
    /// Data version.
    pub(crate) version: f32,
    /// Names for each player.
    player_names: Vec<Option<String>>,
    /// Name IDs for each player.
    player_string_table: Vec<i32>,
    player_base_properties: Vec<PlayerBaseProperties>,
    victory_conquest: bool,
    /// File name of this scenario.
    pub(crate) name: String,
    description_string_table: i32,
    hints_string_table: i32,
    win_message_string_table: i32,
    loss_message_string_table: i32,
    history_string_table: i32,
    scout_string_table: i32,
    description: Option<String>,
    hints: Option<String>,
    win_message: Option<String>,
    loss_message: Option<String>,
    history: Option<String>,
    scout: Option<String>,
    pregame_cinematic: Option<String>,
    victory_cinematic: Option<String>,
    loss_cinematic: Option<String>,
    mission_bmp: Option<String>,
    player_build_lists: Vec<Option<String>>,
    player_city_plans: Vec<Option<String>>,
    player_ai_rules: Vec<Option<String>>,
    player_files: Vec<PlayerFiles>,
    ai_rules_types: Vec<i8>,
}

impl RGEScen {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let version = input.read_f32::<LE>()?;
        let mut player_names = vec![None; 16];
        if version > 1.13 {
            for name in player_names.iter_mut() {
                *name = read_str(input, 256)?;
            }
        }
        let mut player_string_table = vec![-1; 16];
        if version > 1.16 {
            for string_id in player_string_table.iter_mut() {
                *string_id = input.read_i32::<LE>()?;
            }
        }

        let mut player_base_properties = vec![];
        if version > 1.13 {
            for _ in 0..16 {
                let active = input.read_i32::<LE>()?;
                let player_type = input.read_i32::<LE>()?;
                let civilization = input.read_i32::<LE>()?;
                let posture = input.read_i32::<LE>()?;
                player_base_properties.push(PlayerBaseProperties {
                    active,
                    civilization,
                    player_type,
                    posture,
                });
            }
        }

        let victory_conquest = if version >= 1.07 {
            input.read_u8()? != 0
        } else {
            true
        };

        assert_eq!(input.read_i16::<LE>()?, 0, "Unexpected RGE_Timeline");
        assert_eq!(input.read_i16::<LE>()?, 0, "Unexpected RGE_Timeline");
        assert!([-1.0, 0.0].contains(&input.read_f32::<LE>()?));

        let name_length = input.read_i16::<LE>()? as usize;
        let name = read_str(input, name_length)?
            .ok_or_else(|| Error::new(ErrorKind::Other, "must have a file name"))?;

        let (
            description_string_table,
            hints_string_table,
            win_message_string_table,
            loss_message_string_table,
            history_string_table,
        ) = if version >= 1.16 {
            (
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
            )
        } else {
            (-1, -1, -1, -1, -1)
        };

        let scout_string_table = if version >= 1.22 {
            input.read_i32::<LE>()?
        } else {
            -1
        };

        let description_length = input.read_i16::<LE>()? as usize;
        let description = read_str(input, description_length)?;

        let (hints, win_message, loss_message, history) = if version >= 1.11 {
            let hints_length = input.read_i16::<LE>()? as usize;
            let hints = read_str(input, hints_length)?;
            let win_message_length = input.read_i16::<LE>()? as usize;
            let win_message = read_str(input, win_message_length)?;
            let loss_message_length = input.read_i16::<LE>()? as usize;
            let loss_message = read_str(input, loss_message_length)?;
            let history_length = input.read_i16::<LE>()? as usize;
            let history = read_str(input, history_length)?;
            (hints, win_message, loss_message, history)
        } else {
            (None, None, None, None)
        };

        let scout = if version >= 1.22 {
            let scout_length = input.read_i16::<LE>()? as usize;
            read_str(input, scout_length)?
        } else {
            None
        };

        if version < 1.03 {
            // skip some stuff
        }

        let len = input.read_i16::<LE>()? as usize;
        let pregame_cinematic = read_str(input, len)?;
        let len = input.read_i16::<LE>()? as usize;
        let victory_cinematic = read_str(input, len)?;
        let len = input.read_i16::<LE>()? as usize;
        let loss_cinematic = read_str(input, len)?;

        let mission_bmp = if version >= 1.09 {
            let len = input.read_i16::<LE>()? as usize;
            read_str(input, len)?
        } else {
            None
        };

        let _mission_picture = if version >= 1.10 {
            Bitmap::from(input)?
        } else {
            None
        };

        let mut player_build_lists = vec![None; 16];
        for build_list in player_build_lists.iter_mut() {
            let len = input.read_u16::<LE>()? as usize;
            *build_list = read_str(input, len)?;
        }

        let mut player_city_plans = vec![None; 16];
        for city_plan in player_city_plans.iter_mut() {
            let len = input.read_u16::<LE>()? as usize;
            *city_plan = read_str(input, len)?;
        }

        let mut player_ai_rules = vec![None; 16];
        if version >= 1.08 {
            for ai_rules in player_ai_rules.iter_mut() {
                let len = input.read_u16::<LE>()? as usize;
                *ai_rules = read_str(input, len)?;
            }
        }

        let mut player_files = vec![];
        for _ in 0..16 {
            let build_list_length = input.read_i32::<LE>()? as usize;
            let city_plan_length = input.read_i32::<LE>()? as usize;
            let ai_rules_length = if version >= 1.08 {
                input.read_i32::<LE>()? as usize
            } else {
                0
            };

            let build_list = read_str(input, build_list_length)?;
            let city_plan = read_str(input, city_plan_length)?;
            let ai_rules = read_str(input, ai_rules_length)?;

            player_files.push(PlayerFiles {
                build_list,
                city_plan,
                ai_rules,
            });
        }

        let mut ai_rules_types = vec![0; 16];
        if version >= 1.20 {
            for rule_type in ai_rules_types.iter_mut() {
                *rule_type = input.read_i8()?;
            }
        }

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        Ok(RGEScen {
            version,
            player_names,
            player_string_table,
            player_base_properties,
            victory_conquest,
            name,
            description_string_table,
            hints_string_table,
            win_message_string_table,
            loss_message_string_table,
            history_string_table,
            scout_string_table,
            description,
            hints,
            win_message,
            loss_message,
            history,
            scout,
            pregame_cinematic,
            victory_cinematic,
            loss_cinematic,
            mission_bmp,
            player_build_lists,
            player_city_plans,
            player_ai_rules,
            player_files,
            ai_rules_types,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: f32) -> Result<()> {
        output.write_f32::<LE>(version)?;

        if version > 1.13 {
            for name in &self.player_names {
                let mut padded_bytes = vec![0; 256];
                if let Some(ref name) = name {
                    let name_bytes = name.as_bytes();
                    padded_bytes.write_all(name_bytes)?;
                }
                output.write_all(&padded_bytes)?;
            }
        }

        if version > 1.16 {
            for id in &self.player_string_table {
                output.write_i32::<LE>(*id)?;
            }
        }

        if version > 1.13 {
            for props in &self.player_base_properties {
                output.write_i32::<LE>(props.active)?;
                output.write_i32::<LE>(props.player_type)?;
                output.write_i32::<LE>(props.civilization)?;
                output.write_i32::<LE>(props.posture)?;
            }
        }

        if version >= 1.07 {
            output.write_u8(if self.victory_conquest { 1 } else { 0 })?;
        }

        // RGE_Timeline
        output.write_i16::<LE>(0)?;
        output.write_i16::<LE>(0)?;
        output.write_f32::<LE>(-1.0)?;

        write_str(output, &self.name)?;

        if version >= 1.16 {
            output.write_i32::<LE>(self.description_string_table)?;
            output.write_i32::<LE>(self.hints_string_table)?;
            output.write_i32::<LE>(self.win_message_string_table)?;
            output.write_i32::<LE>(self.loss_message_string_table)?;
            output.write_i32::<LE>(self.history_string_table)?;
        }
        if version >= 1.22 {
            output.write_i32::<LE>(self.scout_string_table)?;
        }

        write_opt_str(output, &self.description)?;
        if version >= 1.11 {
            write_opt_str(output, &self.hints)?;
            write_opt_str(output, &self.win_message)?;
            write_opt_str(output, &self.loss_message)?;
            write_opt_str(output, &self.history)?;
        }
        if version >= 1.22 {
            write_opt_str(output, &self.scout)?;
        }

        write_opt_str(output, &self.pregame_cinematic)?;
        write_opt_str(output, &self.victory_cinematic)?;
        write_opt_str(output, &self.loss_cinematic)?;
        if version >= 1.09 {
            // mission_bmp
            write_opt_str(output, &None)?;
        }

        if version >= 1.10 {
            // mission_picture
            output.write_u32::<LE>(0)?;
            output.write_u32::<LE>(0)?;
            output.write_u32::<LE>(0)?;
            output.write_u16::<LE>(1)?;
        }

        for build_list in &self.player_build_lists {
            write_opt_str(output, build_list)?;
        }

        for city_plan in &self.player_city_plans {
            write_opt_str(output, city_plan)?;
        }

        if version >= 1.08 {
            for ai_rules in &self.player_ai_rules {
                write_opt_str(output, ai_rules)?;
            }
        }

        for files in &self.player_files {
            write_opt_i32_str(output, &files.build_list)?;
            write_opt_i32_str(output, &files.city_plan)?;
            if version >= 1.08 {
                write_opt_i32_str(output, &files.ai_rules)?;
            }
        }

        if version >= 1.20 {
            for ai_rules_type in &self.ai_rules_types {
                output.write_i8(*ai_rules_type)?;
            }
        }

        output.write_i32::<LE>(-99)?;

        Ok(())
    }
}

#[derive(Debug)]
struct PlayerStartResources {
    pub(crate) gold: i32,
    pub(crate) wood: i32,
    pub(crate) food: i32,
    pub(crate) stone: i32,
    pub(crate) ore: i32,
    pub(crate) goods: i32,
    pub(crate) player_color: Option<i32>,
}

impl PlayerStartResources {
    pub fn from<R: Read>(input: &mut R, version: f32) -> Result<Self> {
        Ok(Self {
            gold: input.read_i32::<LE>()?,
            wood: input.read_i32::<LE>()?,
            food: input.read_i32::<LE>()?,
            stone: input.read_i32::<LE>()?,
            ore: if version >= 1.17 {
                input.read_i32::<LE>()?
            } else { 100 },
            goods: if version >= 1.17 {
                input.read_i32::<LE>()?
            } else { 0 },
            player_color: if version >= 1.24 {
                Some(input.read_i32::<LE>()?)
            } else { None },
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: f32) -> Result<()> {
        output.write_i32::<LE>(self.gold)?;
        output.write_i32::<LE>(self.wood)?;
        output.write_i32::<LE>(self.food)?;
        output.write_i32::<LE>(self.stone)?;
        if version >= 1.17 {
            output.write_i32::<LE>(self.ore)?;
            output.write_i32::<LE>(self.goods)?;
        }
        if version >= 1.24 {
            output.write_i32::<LE>(self.player_color.unwrap_or(0))?;
        }
        Ok(())
    }
}

/// Initial player attributes.
#[derive(Debug)]
struct WorldPlayerData {
    /// Initial food count.
    pub(crate) food: f32,
    /// Initial wood count.
    pub(crate) wood: f32,
    /// Initial gold count.
    pub(crate) gold: f32,
    /// Initial stone count.
    pub(crate) stone: f32,
    /// Initial ore count. (unused, except in some mods)
    pub(crate) ore: f32,
    /// Initial trade goods count. (unused)
    pub(crate) goods: f32,
    /// Max population.
    pub(crate) population: f32,
}

impl WorldPlayerData {
    pub fn from<R: Read>(input: &mut R, version: f32) -> Result<Self> {
        Ok(Self {
            wood: if version > 1.06 { input.read_f32::<LE>()? } else { 200.0 },
            food: if version > 1.06 { input.read_f32::<LE>()? } else { 200.0 },
            gold: if version > 1.06 { input.read_f32::<LE>()? } else { 50.0 },
            stone: if version > 1.06 { input.read_f32::<LE>()? } else { 100.0 },
            ore: if version > 1.12 { input.read_f32::<LE>()? } else { 100.0 },
            goods: if version > 1.12 { input.read_f32::<LE>()? } else { 0.0 },
            population: if version >= 1.14 { input.read_f32::<LE>()? } else { 75.0 } 
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: f32) -> Result<()> {
        output.write_f32::<LE>(self.gold)?;
        output.write_f32::<LE>(self.wood)?;
        output.write_f32::<LE>(self.food)?;
        output.write_f32::<LE>(self.stone)?;
        if version > 1.12 {
            output.write_f32::<LE>(self.ore)?;
        }
        if version > 1.12 {
            output.write_f32::<LE>(self.goods)?;
        }
        if version >= 1.14 {
            output.write_f32::<LE>(self.population)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct VictoryEntry {
    command: i8,
    object_type: i32,
    player_id: i32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    number: i32,
    count: i32,
    source_object: i32,
    target_object: i32,
    victory_group: i8,
    ally_flag: i8,
    state: i8,
}

impl VictoryEntry {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let command = input.read_i8()?;
        let object_type = input.read_i32::<LE>()?;
        let player_id = input.read_i32::<LE>()?;
        let x0 = input.read_f32::<LE>()?;
        let y0 = input.read_f32::<LE>()?;
        let x1 = input.read_f32::<LE>()?;
        let y1 = input.read_f32::<LE>()?;
        let number = input.read_i32::<LE>()?;
        let count = input.read_i32::<LE>()?;
        let source_object = input.read_i32::<LE>()?;
        let target_object = input.read_i32::<LE>()?;
        let victory_group = input.read_i8()?;
        let ally_flag = input.read_i8()?;
        let state = input.read_i8()?;

        Ok(Self {
            command,
            object_type,
            player_id,
            x0,
            y0,
            x1,
            y1,
            number,
            count,
            source_object,
            target_object,
            victory_group,
            ally_flag,
            state,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i8(self.command)?;
        output.write_i32::<LE>(self.object_type)?;
        output.write_i32::<LE>(self.player_id)?;
        output.write_f32::<LE>(self.x0)?;
        output.write_f32::<LE>(self.y0)?;
        output.write_f32::<LE>(self.x1)?;
        output.write_f32::<LE>(self.y1)?;
        output.write_i32::<LE>(self.number)?;
        output.write_i32::<LE>(self.count)?;
        output.write_i32::<LE>(self.source_object)?;
        output.write_i32::<LE>(self.target_object)?;
        output.write_i8(self.victory_group)?;
        output.write_i8(self.ally_flag)?;
        output.write_i8(self.state)?;
        Ok(())
    }
}

#[derive(Debug)]
struct VictoryPointEntry {
    command: i8,
    state: i8,
    attribute: i32,
    amount: i32,
    points: i32,
    current_points: i32,
    id: i8,
    group: i8,
    current_attribute_amount: f32,
    attribute1: i32,
    current_attribute_amount1: f32,
}

impl VictoryPointEntry {
    pub fn from<R: Read>(input: &mut R, version: f32) -> Result<Self> {
        let command = input.read_i8()?;
        let state = input.read_i8()?;
        let attribute = input.read_i32::<LE>()?;
        let amount = input.read_i32::<LE>()?;
        let points = input.read_i32::<LE>()?;
        let current_points = input.read_i32::<LE>()?;
        let id = input.read_i8()?;
        let group = input.read_i8()?;
        let current_attribute_amount = input.read_f32::<LE>()?;
        let (attribute1, current_attribute_amount1) = if version >= 2.0 {
            (input.read_i32::<LE>()?, input.read_f32::<LE>()?)
        } else {
            (-1, 0.0)
        };

        Ok(Self {
            command,
            state,
            attribute,
            amount,
            points,
            current_points,
            id,
            group,
            current_attribute_amount,
            attribute1,
            current_attribute_amount1,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: f32) -> Result<()> {
        output.write_i8(self.command)?;
        output.write_i8(self.state)?;
        output.write_i32::<LE>(self.attribute)?;
        output.write_i32::<LE>(self.amount)?;
        output.write_i32::<LE>(self.points)?;
        output.write_i32::<LE>(self.current_points)?;
        output.write_i8(self.id)?;
        output.write_i8(self.group)?;
        output.write_f32::<LE>(self.current_attribute_amount)?;
        if version >= 2.0 {
            output.write_i32::<LE>(self.attribute1)?;
            output.write_f32::<LE>(self.current_attribute_amount1)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct VictoryConditions {
    version: f32,
    victory: bool,
    total_points: i32,
    starting_points: i32,
    starting_group: i32,
    entries: Vec<VictoryEntry>,
    point_entries: Vec<VictoryPointEntry>,
}

impl VictoryConditions {
    pub fn from<R: Read>(input: &mut R, has_version: bool) -> Result<Self> {
        let version = if has_version {
            input.read_f32::<LE>()?
        } else {
            0.0
        };

        let num_conditions = input.read_i32::<LE>()?;
        let victory = input.read_u8()? != 0;

        let mut entries = Vec::with_capacity(num_conditions as usize);
        for _ in 0..num_conditions {
            entries.push(VictoryEntry::from(input)?);
        }

        let mut total_points = 0;
        let mut point_entries = vec![];
        let mut starting_points = 0;
        let mut starting_group = 0;

        if version >= 1.0 {
            total_points = input.read_i32::<LE>()?;
            let num_point_entries = input.read_i32::<LE>()?;

            if version >= 2.0 {
                starting_points = input.read_i32::<LE>()?;
                starting_group = input.read_i32::<LE>()?;
            }

            for _ in 0..num_point_entries {
                point_entries.push(VictoryPointEntry::from(input, version)?);
            }
        }

        Ok(Self {
            version,
            victory,
            total_points,
            starting_points,
            starting_group,
            entries,
            point_entries,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: Option<f32>) -> Result<()> {
        if let Some(v) = version {
            output.write_f32::<LE>(v)?;
        }

        let version = version.unwrap_or(std::f32::MIN);

        output.write_i32::<LE>(self.entries.len() as i32)?;
        output.write_u8(if self.victory { 1 } else { 0 })?;

        for entry in &self.entries {
            entry.write_to(output)?;
        }

        if version >= 1.0 {
            output.write_i32::<LE>(self.total_points)?;
            output.write_i32::<LE>(self.point_entries.len() as i32)?;

            if version >= 2.0 {
                output.write_i32::<LE>(self.starting_points)?;
                output.write_i32::<LE>(self.starting_group)?;
            }

            for entry in &self.point_entries {
                entry.write_to(output, version)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct ScenarioPlayerData {
    name: Option<String>,
    view: (f32, f32),
    location: (i16, i16),
    allied_victory: bool,
    relations: Vec<i8>,
    unit_diplomacy: Vec<i32>,
    color: Option<i32>,
    victory: VictoryConditions,
}

impl ScenarioPlayerData {
    pub fn from<R: Read>(input: &mut R, version: f32) -> Result<Self> {
        let len = input.read_u16::<LE>()?;
        let name = read_str(input, len as usize)?;

        let view = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );

        let location = (
            input.read_i16::<LE>()?,
            input.read_i16::<LE>()?,
        );

        let allied_victory = if version > 1.0 {
            input.read_u8()? != 0
        } else {
            false
        };

        let diplo_count = input.read_i16::<LE>()?;
        let mut relations = Vec::with_capacity(diplo_count as usize);
        for _ in 0..diplo_count {
            relations.push(input.read_i8()?);
        }

        let unit_diplomacy = if version >= 1.08 {
            vec![
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
            ]
        } else {
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0]
        };

        let color = if version >= 1.13 {
            Some(input.read_i32::<LE>()?)
        } else {
            None
        };

        let victory = VictoryConditions::from(input, version >= 1.09)?;

        Ok(ScenarioPlayerData {
            name,
            view,
            location,
            allied_victory,
            relations,
            unit_diplomacy,
            color,
            victory,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: f32, victory_version: f32) -> Result<()> {
        write_opt_str(output, &self.name)?;

        output.write_f32::<LE>(self.view.0)?;
        output.write_f32::<LE>(self.view.1)?;

        output.write_i16::<LE>(self.location.0)?;
        output.write_i16::<LE>(self.location.1)?;

        if version > 1.0 {
            output.write_u8(if self.allied_victory { 1 } else { 0 })?;
        };

        output.write_i16::<LE>(self.relations.len() as i16)?;
        for rel in &self.relations {
            output.write_i8(*rel)?;
        }

        if version >= 1.08 {
            output.write_i32::<LE>(self.unit_diplomacy[0])?;
            output.write_i32::<LE>(self.unit_diplomacy[1])?;
            output.write_i32::<LE>(self.unit_diplomacy[2])?;
            output.write_i32::<LE>(self.unit_diplomacy[3])?;
            output.write_i32::<LE>(self.unit_diplomacy[4])?;
            output.write_i32::<LE>(self.unit_diplomacy[5])?;
            output.write_i32::<LE>(self.unit_diplomacy[6])?;
            output.write_i32::<LE>(self.unit_diplomacy[7])?;
            output.write_i32::<LE>(self.unit_diplomacy[8])?;
        }

        if version >= 1.13 {
            output.write_i32::<LE>(self.color.unwrap_or(-1))?;
        }

        self.victory.write_to(output, if version >= 1.09 {
            Some(victory_version)
        } else {
            None
        })?;

        Ok(())
    }
}

#[derive(Debug)]
struct VictoryInfo {
    pub(crate) conquest: i32,
    pub(crate) ruins: i32,
    pub(crate) artifacts: i32,
    pub(crate) discoveries: i32,
    pub(crate) exploration: i32,
    pub(crate) gold: i32,
}

impl VictoryInfo {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        Ok(Self {
            conquest: input.read_i32::<LE>()?,
            ruins: input.read_i32::<LE>()?,
            artifacts: input.read_i32::<LE>()?,
            discoveries: input.read_i32::<LE>()?,
            exploration: input.read_i32::<LE>()?,
            gold: input.read_i32::<LE>()?,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.conquest)?;
        output.write_i32::<LE>(self.ruins)?;
        output.write_i32::<LE>(self.artifacts)?;
        output.write_i32::<LE>(self.discoveries)?;
        output.write_i32::<LE>(self.exploration)?;
        output.write_i32::<LE>(self.gold)?;

        Ok(())
    }
}

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

        Ok(Self { width, height, tiles })
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
        self.tiles.get(y as usize)
            .and_then(|row| row.get(x as usize))
    }

    pub fn tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        self.tiles.get_mut(y as usize)
            .and_then(|row| row.get_mut(x as usize))
    }

    pub fn tiles(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter()
            .map(|row| row.iter())
            .flatten()
    }

    pub fn tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile> {
        self.tiles.iter_mut()
            .map(|row| row.iter_mut())
            .flatten()
    }
}

#[derive(Debug)]
pub struct ScenarioObject {
    /// Position (x, y, z) of this object.
    pub position: (f32, f32, f32),
    /// This object's unique ID.
    pub id: i32,
    /// The type ID of this object.
    pub object_type: i16,
    /// State value.
    pub state: u8,
    /// Radian angle this unit is facing.
    pub angle: f32,
    /// Current animation frame.
    pub frame: i16,
    /// ID of the object this object is garrisoned in, or -1 when not
    /// garrisoned.
    pub garrisoned_in: Option<i32>,
}

impl ScenarioObject {
    pub fn from<R: Read>(input: &mut R, version: SCXVersion) -> Result<Self> {
        let position = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        let id = input.read_i32::<LE>()?;
        let object_type = input.read_i16::<LE>()?;
        let state = input.read_u8()?;
        let angle = input.read_f32::<LE>()?;
        let frame = if cmp_scx_version(version, *b"1.15") == Ordering::Less {
            -1
        } else {
            input.read_i16::<LE>()?
        };
        let garrisoned_in = if cmp_scx_version(version, *b"1.13") == Ordering::Less {
            None
        } else {
            Some(input.read_i32::<LE>()?)
        }.and_then(|id| match id {
            -1 => None,
            id => Some(id),
        })
        .and_then(|id| match id {
            // 0 means -1 in more recent versions
            0 if &version == b"1.21" || &version == b"1.22" => None,
            id => Some(id),
        });

        Ok(Self {
            position,
            id,
            object_type,
            state,
            angle,
            frame,
            garrisoned_in,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: SCXVersion) -> Result<()> {
        output.write_f32::<LE>(self.position.0)?;
        output.write_f32::<LE>(self.position.1)?;
        output.write_f32::<LE>(self.position.2)?;
        output.write_i32::<LE>(self.id)?;
        output.write_i16::<LE>(self.object_type)?;
        output.write_u8(self.state)?;
        output.write_f32::<LE>(self.angle)?;
        if cmp_scx_version(version, *b"1.14") == Ordering::Greater {
            output.write_i16::<LE>(self.frame)?;
        }
        if cmp_scx_version(version, *b"1.12") == Ordering::Greater {
            match self.garrisoned_in {
                Some(id) => output.write_i32::<LE>(id)?,
                None => output.write_i32::<LE>(-1)?,
            }
        }
        Ok(())
    }
}

/// AoE1's victory info.
///
/// This was replaced by VictoryConditions in AoE2.
#[derive(Debug, Clone, Default)]
pub struct LegacyVictoryInfo {
    pub object_type: i32,
    pub all_flag: bool,
    pub player_id: i32,
    pub dest_object_id: i32,
    pub area: (f32, f32, f32, f32),
    pub victory_type: i32,
    pub amount: i32,
    pub attribute: i32,
    pub object_id: i32,
    pub dest_object_id2: i32,
}

impl LegacyVictoryInfo {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let object_type = input.read_i32::<LE>()?;
        let all_flag = input.read_i32::<LE>()? != 0;
        let player_id = input.read_i32::<LE>()?;
        let dest_object_id = input.read_i32::<LE>()?;
        let area = (
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
            input.read_f32::<LE>()?,
        );
        let victory_type = input.read_i32::<LE>()?;
        let amount = input.read_i32::<LE>()?;
        let attribute = input.read_i32::<LE>()?;
        let object_id = input.read_i32::<LE>()?;
        let dest_object_id2 = input.read_i32::<LE>()?;
        // Should be 0 because they're pointers
        let _object = input.read_u32::<LE>()?;
        let _dest_object = input.read_u32::<LE>()?;

        Ok(Self {
            object_type,
            all_flag,
            player_id,
            dest_object_id,
            area,
            victory_type,
            amount,
            attribute,
            object_id,
            dest_object_id2,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.object_type)?;
        output.write_i32::<LE>(if self.all_flag { 1 } else { 0 })?;
        output.write_i32::<LE>(self.player_id)?;
        output.write_i32::<LE>(self.dest_object_id)?;
        output.write_f32::<LE>(self.area.0)?;
        output.write_f32::<LE>(self.area.1)?;
        output.write_f32::<LE>(self.area.2)?;
        output.write_f32::<LE>(self.area.3)?;
        output.write_i32::<LE>(self.victory_type)?;
        output.write_i32::<LE>(self.amount)?;
        output.write_i32::<LE>(self.attribute)?;
        output.write_i32::<LE>(self.object_id)?;
        output.write_i32::<LE>(self.dest_object_id2)?;
        output.write_u32::<LE>(0)?;
        output.write_u32::<LE>(0)?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct TribeScen {
    /// "Engine" data.
    ///
    /// This distinction doesn't make much sense as a user of this library, but
    /// it exists internally in AoC and affects the storage format (eg.  some
    /// things are duplicate).
    pub(crate) base: RGEScen,
    /// Starting resources for players.
    player_start_resources: Vec<PlayerStartResources>,
    /// Victory settings.
    victory: VictoryInfo,
    /// Whether all victory conditions need to be met for victory to occur.
    victory_all_flag: bool,
    /// Type of victory condition to use in multiplayer games.
    mp_victory_type: i32,
    /// Required score to attain multiplayer victory.
    victory_score: i32,
    /// Time at which the highest-scoring player will win the multiplayer match.
    victory_time: i32,
    /// Initial diplomacy stances between players.
    diplomacy: Vec<Vec<DiplomaticStance>>,
    legacy_victory_info: Vec<Vec<LegacyVictoryInfo>>,
    /// Whether Allied Victory is enabled for each player.
    allied_victory: Vec<i32>,
    teams_locked: bool,
    can_change_teams: bool,
    random_start_locations: bool,
    max_teams: u8,
    /// Number of disabled techs per player.
    num_disabled_techs: Vec<i32>,
    /// Disabled tech IDs per player.
    disabled_techs: Vec<Vec<i32>>,
    /// Number of disabled units per player.
    num_disabled_units: Vec<i32>,
    /// Disabled unit IDs per player.
    disabled_units: Vec<Vec<i32>>,
    /// Number of disabled buildings per player.
    num_disabled_buildings: Vec<i32>,
    /// Disabled building IDs per player.
    disabled_buildings: Vec<Vec<i32>>,
    /// Some unknown scenario option...
    unknown_scenario_option: i32,
    /// Some unknown scenario option...
    unknown_scenario_option_2: i32,
    /// Whether "All Techs" is enabled.
    all_techs: bool,
    /// The starting age per player.
    player_start_ages: Vec<StartingAge>,
    /// The initial camera location.
    view: (i32, i32),
    /// The map type.
    map_type: Option<i32>,
    base_priorities: Vec<i8>,
}

impl TribeScen {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut base = RGEScen::from(input)?;
        let version = base.version;

        let mut player_start_resources = vec![];

        // Moved to RGEScen in 1.13
        if version <= 1.13 {
            for name in base.player_names.iter_mut() {
                *name = read_str(input, 256)?;
            }

            for _ in 0..16 {
                let active = input.read_i32::<LE>()?;
                let resources = PlayerStartResources::from(input, version)?;
                let player_type = input.read_i32::<LE>()?;
                let civilization = input.read_i32::<LE>()?;
                let posture = input.read_i32::<LE>()?;
                player_start_resources.push(resources);
                base.player_base_properties.push(PlayerBaseProperties {
                    active,
                    civilization,
                    player_type,
                    posture,
                });
            }
        } else {
            for _ in 0..16 {
                player_start_resources.push(PlayerStartResources::from(input, version)?);
            }
        }

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        let victory = VictoryInfo::from(input)?;
        let victory_all_flag = input.read_i32::<LE>()? != 0;

        let mp_victory_type = if version >= 1.13 {
            input.read_i32::<LE>()?
        } else {
            4
        };
        let victory_score = if version >= 1.13 {
            input.read_i32::<LE>()?
        } else {
            900
        };
        let victory_time = if version >= 1.13 {
            input.read_i32::<LE>()?
        } else {
            9000
        };

        let mut diplomacy = vec![];
        for _ in 0..16 {
            let mut player_diplomacy = vec![];
            for _ in 0..16 {
                player_diplomacy.push(DiplomaticStance::try_from(
                        input.read_i32::<LE>()?)?);
            }
            diplomacy.push(player_diplomacy);
        }

        let mut legacy_victory_info = vec![vec![]; 16];
        for list in legacy_victory_info.iter_mut() {
            for _ in 0..12 {
                list.push(LegacyVictoryInfo::from(input)?);
                if list.last().unwrap().victory_type != 0 {
                    dbg!(list.last());
                }
            }
        }

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        let mut allied_victory = vec![];
        for _ in 0..16 {
            allied_victory.push(input.read_i32::<LE>()?);
        }

        let (
            teams_locked,
            can_change_teams,
            random_start_locations,
            max_teams,
        ) = if version >= 1.24 {
            (
                input.read_i8()? != 0,
                input.read_i8()? != 0,
                input.read_i8()? != 0,
                input.read_u8()?,
            )
        } else if cmp_float!(version == 1.23) {
            (
                input.read_i32::<LE>()? != 0,
                true,
                true,
                4,
            )
        } else {
            (false, true, true, 4)
        };

        let mut num_disabled_techs = vec![0; 16];
        let mut disabled_techs = vec![vec![]; 16];
        let mut num_disabled_units = vec![0; 16];
        let mut disabled_units = vec![vec![]; 16];
        let mut num_disabled_buildings = vec![0; 16];
        let mut disabled_buildings = vec![vec![]; 16];

        if version >= 1.18 {
            for num in num_disabled_techs.iter_mut() {
                *num = input.read_i32::<LE>()?;
            }
            for player_disabled_techs in disabled_techs.iter_mut() {
                for _ in 0..30 {
                    player_disabled_techs.push(input.read_i32::<LE>()?);
                }
            }

            for num in num_disabled_units.iter_mut() {
                *num = input.read_i32::<LE>()?;
            }
            for player_disabled_units in disabled_units.iter_mut() {
                for _ in 0..30 {
                    player_disabled_units.push(input.read_i32::<LE>()?);
                }
            }

            for num in num_disabled_buildings.iter_mut() {
                *num = input.read_i32::<LE>()?;
            }
            let max_disabled_buildings = if version >= 1.25 { 30 } else { 20 };
            for player_disabled_buildings in disabled_buildings.iter_mut() {
                for _ in 0..max_disabled_buildings {
                    player_disabled_buildings.push(input.read_i32::<LE>()?);
                }
            }
        } else if version > 1.03 {
            // Old scenarios only allowed disabling up to 20 techs per player.
            for i in 0..16 {
                let player_disabled_techs = &mut disabled_techs[i];
                for _ in 0..20 {
                    player_disabled_techs.push(input.read_i32::<LE>()?);
                }
                // The number of disabled techs wasn't stored either, so we need to guess it!
                num_disabled_techs[i] = player_disabled_techs.iter()
                    .position(|val| *val <= 0)
                    .map(|index| (index as i32) + 1)
                    .unwrap_or(0);
            }
        } else {
            // <= 1.03 did not support disabling anything
        }

        let unknown_scenario_option = if version > 1.04 {
            input.read_i32::<LE>()?
        } else {
            0
        };
        let (unknown_scenario_option_2, all_techs) = if version >= 1.12 {
            (
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()? != 0,
            )
        } else {
            (0, false)
        };

        let mut player_start_ages = vec![StartingAge::Default; 16];
        if version > 1.05 {
            for start_age in player_start_ages.iter_mut() {
                *start_age = StartingAge::try_from(input.read_i32::<LE>()?, version)?;
            }
        }

        if version >= 1.02 {
            let sep = input.read_i32::<LE>()?;
            debug_assert_eq!(sep, -99);
        }

        let view = if version >= 1.19 {
            (
                input.read_i32::<LE>()?,
                input.read_i32::<LE>()?,
            )
        } else {
            (-1, -1)
        };

        let map_type = if version >= 1.21 {
            Some(input.read_i32::<LE>()?).and_then(|v| {
                if v != -1 {
                    Some(v)
                } else {
                    None
                }
            })
        } else {
            None
        };

        let mut base_priorities = vec![0; 16];
        if version >= 1.24 {
            for priority in base_priorities.iter_mut() {
                *priority = input.read_i8()?;
            }
        }

        Ok(TribeScen {
            base,
            player_start_resources,
            victory,
            victory_all_flag,
            mp_victory_type,
            victory_score,
            victory_time,
            diplomacy,
            legacy_victory_info,
            allied_victory,
            teams_locked,
            can_change_teams,
            random_start_locations,
            max_teams,
            num_disabled_techs,
            disabled_techs,
            num_disabled_units,
            disabled_units,
            num_disabled_buildings,
            disabled_buildings,
            unknown_scenario_option,
            unknown_scenario_option_2,
            all_techs,
            player_start_ages,
            view,
            map_type,
            base_priorities,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: f32) -> Result<()> {
        self.base.write_to(output, version)?;

        if version <= 1.13 {
            for name in &self.base.player_names {
                let mut padded_bytes = vec![0; 256];
                if let Some(ref name) = name {
                    let name_bytes = name.as_bytes();
                    padded_bytes.write_all(name_bytes)?;
                }
                output.write_all(&padded_bytes)?;
            }

            for i in 0..16 {
                let properties = &self.base.player_base_properties[i];
                let resources = &self.player_start_resources[i];
                output.write_i32::<LE>(properties.active)?;
                resources.write_to(output, version)?;
                output.write_i32::<LE>(properties.player_type)?;
                output.write_i32::<LE>(properties.civilization)?;
                output.write_i32::<LE>(properties.posture)?;
            }
        } else {
            for start_resources in &self.player_start_resources {
                start_resources.write_to(output, version)?;
            }
        }

        if version >= 1.02 {
            output.write_i32::<LE>(-99)?;
        }

        self.victory.write_to(output)?;
        output.write_i32::<LE>(if self.victory_all_flag { 1 } else { 0 })?;

        if version >= 1.13 {
            output.write_i32::<LE>(self.mp_victory_type)?;
            output.write_i32::<LE>(self.victory_score)?;
            output.write_i32::<LE>(self.victory_time)?;
        }

        for player_diplomacy in &self.diplomacy {
            for stance in player_diplomacy {
                output.write_i32::<LE>((*stance).into())?;
            }
        }

        for list in &self.legacy_victory_info {
            for entry in list {
                entry.write_to(output)?;
            }
        }

        if version >= 1.02 {
            output.write_i32::<LE>(-99)?;
        }

        for value in &self.allied_victory {
            output.write_i32::<LE>(*value)?;
        }

        if version >= 1.24 {
            output.write_i8(if self.teams_locked { 1 } else { 0 })?;
            output.write_i8(if self.can_change_teams { 1 } else { 0 })?;
            output.write_i8(if self.random_start_locations { 1 } else { 0 })?;
            output.write_u8(self.max_teams)?;
        } else if cmp_float!(version == 1.23) {
            output.write_i32::<LE>(if self.teams_locked { 1 } else { 0 })?;
        }

        let max_disabled_buildings = if version >= 1.25 { 30 } else { 20 };
        if version >= 1.18 {
            let most = *self.num_disabled_buildings.iter().max().unwrap_or(&0);
            if most > max_disabled_buildings {
                return Err(Error::new(ErrorKind::Other,
                      format!("too many disabled buildings: got {}, but requested version supports up to {}", most, max_disabled_buildings)));
            }

            for num in &self.num_disabled_techs {
                output.write_i32::<LE>(*num)?;
            }
            for player_disabled_techs in &self.disabled_techs {
                for i in 0..30 {
                    output.write_i32::<LE>(*player_disabled_techs.get(i).unwrap_or(&-1))?;
                }
            }

            for num in &self.num_disabled_units {
                output.write_i32::<LE>(*num)?;
            }
            for player_disabled_units in &self.disabled_units {
                for i in 0..30 {
                    output.write_i32::<LE>(*player_disabled_units.get(i).unwrap_or(&-1))?;
                }
            }

            for num in &self.num_disabled_buildings {
                output.write_i32::<LE>(*num)?;
            }
            for player_disabled_buildings in &self.disabled_buildings {
                for i in 0..max_disabled_buildings as usize {
                    output.write_i32::<LE>(*player_disabled_buildings.get(i).unwrap_or(&-1))?;
                }
            }
        } else if version > 1.03 {
            let most = *self.num_disabled_techs.iter().max().unwrap_or(&0);
            if most > 20 {
                return Err(Error::new(ErrorKind::Other,
                      format!("too many disabled techs: got {}, but requested version supports up to 20", most)));
            }
            if self.num_disabled_units.iter().any(|&n| n > 0) {
                return Err(Error::new(ErrorKind::Other,
                      format!("requested version does not support disabling units")));
            }
            if self.num_disabled_buildings.iter().any(|&n| n > 0) {
                return Err(Error::new(ErrorKind::Other,
                      format!("requested version does not support disabling buildings")));
            }

            // Old scenarios only allowed disabling up to 20 techs per player.
            for player_disabled_techs in &self.disabled_techs {
                for i in 0..20 {
                    output.write_i32::<LE>(*player_disabled_techs.get(i).unwrap_or(&-1))?;
                }
            }
        } else {
            // <= 1.03 did not support disabling anything
            if self.num_disabled_techs.iter().any(|&n| n > 0) {
                return Err(Error::new(ErrorKind::Other,
                      format!("requested version does not support disabling techs")));
            }
            if self.num_disabled_units.iter().any(|&n| n > 0) {
                return Err(Error::new(ErrorKind::Other,
                      format!("requested version does not support disabling units")));
            }
            if self.num_disabled_buildings.iter().any(|&n| n > 0) {
                return Err(Error::new(ErrorKind::Other,
                      format!("requested version does not support disabling buildings")));
            }
        }

        if version > 1.04 {
            output.write_i32::<LE>(0)?;
        }
        if version >= 1.12 {
            output.write_i32::<LE>(0)?;
            output.write_i32::<LE>(if self.all_techs { 1 } else { 0 })?;
        }

        if version > 1.05 {
            for start_age in &self.player_start_ages {
                output.write_i32::<LE>(start_age.to_i32(version))?;
            }
        }

        if version >= 1.02 {
            output.write_i32::<LE>(-99)?;
        }

        if version >= 1.19 {
            output.write_i32::<LE>(self.view.0)?;
            output.write_i32::<LE>(self.view.1)?;
        }

        if version >= 1.21 {
            output.write_i32::<LE>(self.map_type.unwrap_or(-1))?;
        }

        if version >= 1.24 {
            for priority in &self.base_priorities {
                output.write_i8(*priority)?;
            }
        }

        Ok(())
    }

    pub fn version(&self) -> f32 {
        self.base.version
    }

    pub fn description(&self) -> Option<&str> {
        // Convert String to &str: https://stackoverflow.com/a/31234028
        self.base.description.as_ref()
            .map(|s| &**s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
enum AIErrorCode {
    ConstantAlreadyDefined = 0x0,
    FileOpenFailed = 0x1,
    FileReadFailed = 0x2,
    InvalidIdentifier = 0x3,
    InvalidKeyword = 0x4,
    InvalidPreprocessorDirective = 0x5,
    ListFull = 0x6,
    MissingArrow = 0x7,
    MissingClosingParenthesis = 0x8,
    MissingClosingQuote = 0x9,
    MissingEndIf = 0xA,
    MissingFileName = 0xB,
    MissingIdentifier = 0xC,
    MissingKeyword = 0xD,
    MissingLHS = 0xE,
    MissingOpeningParenthesis = 0xF,
    MissingPreprocessorSymbol = 0x10,
    MissingRHS = 0x11,
    NoRules = 0x12,
    PreprocessorNestingTooDeep = 0x13,
    RuleTooLong = 0x14,
    StringTableFull = 0x15,
    UndocumentedError = 0x16,
    UnexpectedElse = 0x17,
    UnexpectedEndIf = 0x18,
    UnexpectedError = 0x19,
    UnexpectedEOF = 0x1A,
}

#[derive(Debug, Clone)]
pub struct AIErrorInfo {
    filename: String,
    line_number: i32,
    description: String,
    error_code: AIErrorCode,
}

fn parse_bytes(bytes: &[u8]) -> Result<String> {
    let mut bytes = bytes.to_vec();
    if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
        bytes.truncate(end);
    }
    if bytes.is_empty() {
        Ok("<empty>".to_string())
    } else {
        String::from_utf8(bytes)
            .map_err(|_| Error::new(ErrorKind::Other, "invalid string"))
    }
}

impl AIErrorInfo {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let mut filename_bytes = [0; 257];
        input.read_exact(&mut filename_bytes)?;
        let line_number = input.read_i32::<LE>()?;
        let mut description_bytes = [0; 128];
        input.read_exact(&mut description_bytes)?;
        let error_code = AIErrorCode::from_u32(input.read_u32::<LE>()?).unwrap();

        let filename = parse_bytes(&filename_bytes)?;
        let description = parse_bytes(&description_bytes)?;

        Ok(AIErrorInfo {
            filename,
            line_number,
            description,
            error_code,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AIFile {
    filename: String,
    content: String,
}

impl AIFile {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
        let len = input.read_i32::<LE>()? as usize;
        let filename = read_str(input, len)?.expect("missing ai file name");
        let len = input.read_i32::<LE>()? as usize;
        let content = read_str(input, len)?.expect("empty ai file?");

        Ok(Self { filename, content })
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIInfo {
    error: Option<AIErrorInfo>,
    files: Vec<AIFile>,
}

impl AIInfo {
    pub fn from<R: Read>(input: &mut R) -> Result<Option<Self>> {
        let has_ai_files = input.read_u32::<LE>()? != 0;
        let has_error = input.read_u32::<LE>()? != 0;

        if !has_error && !has_ai_files {
            return Ok(None);
        }

        let error = if has_error {
            Some(AIErrorInfo::from(input)?)
        } else {
            None
        };

        let num_ai_files = input.read_u32::<LE>()?;
        let mut files = vec![];
        for _ in 0..num_ai_files {
            files.push(AIFile::from(input)?);
        }

        Ok(Some(Self { error, files }))
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u32::<LE>(0)?;
        output.write_u32::<LE>(0)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SCXFormat {
    /// Version of the SCX format.
    pub(crate) version: SCXVersion,
    /// Uncompressed header containing metadata for display.
    pub(crate) header: SCXHeader,
    /// ID for the next-placed/created object.
    pub(crate) next_object_id: i32,
    /// Scenario data.
    pub(crate) tribe_scen: TribeScen,
    /// Map data.
    pub(crate) map: Map,
    /// Player data.
    world_players: Vec<WorldPlayerData>,
    /// Objects data.
    pub(crate) player_objects: Vec<Vec<ScenarioObject>>,
    /// Player data.
    scenario_players: Vec<ScenarioPlayerData>,
    /// Triggers (only in AoK and up).
    triggers: Option<TriggerSystem>,
    /// AI information (AoK and up).
    ai_info: Option<AIInfo>,
}

impl SCXFormat {
    /// Extract version bundle information from a parsed SCX file.
    pub fn version(&self) -> VersionBundle {
        VersionBundle {
            format: self.version,
            header: self.header.version,
            data: self.tribe_scen.version(),
            ..VersionBundle::aoc()
        }
    }

    fn load_121<R: Read>(version: SCXVersion, player_version: f32, input: &mut R) -> Result<Self> {
        let header = SCXHeader::from(input, version)?;

        let mut input = DeflateDecoder::new(input);
        let next_object_id = input.read_i32::<LE>()?;

        let tribe_scen = TribeScen::from(&mut input)?;

        let map = Map::from(&mut input)?;

        let num_players = input.read_i32::<LE>()?;
        let mut world_players = vec![];
        for _ in 1..num_players {
            world_players.push(WorldPlayerData::from(&mut input, player_version)?);
        }

        let mut player_objects = vec![];
        for _ in 0..num_players {
            let mut objects = vec![];
            let num_objects = input.read_u32::<LE>()?;
            for _ in 0..num_objects {
                objects.push(ScenarioObject::from(&mut input, version)?);
            }
            player_objects.push(objects);
        }

        let num_scenario_players = input.read_i32::<LE>()?;
        let mut scenario_players = vec![];
        for _ in 1..num_scenario_players {
            scenario_players.push(ScenarioPlayerData::from(&mut input, player_version)?);
        }

        let triggers = if cmp_scx_version(version, *b"1.14") == Ordering::Less {
            None
        } else {
            Some(TriggerSystem::from(&mut input)?)
        };

        let ai_info = if cmp_scx_version(version, *b"1.17") == Ordering::Greater && cmp_scx_version(version, *b"2.00") == Ordering::Less {
            AIInfo::from(&mut input)?
        } else {
            None
        };

        Ok(SCXFormat {
            version,
            header,
            next_object_id,
            tribe_scen,
            map,
            world_players,
            player_objects,
            scenario_players,
            triggers,
            ai_info,
        })
    }

    pub fn load_scenario<R: Read>(input: &mut R) -> Result<Self> {
        let mut format_version = [0; 4];
        input.read_exact(&mut format_version)?;
        match &format_version {
            b"1.01" => unimplemented!(),
            b"1.02" => unimplemented!(),
            b"1.03" => unimplemented!(),
            b"1.04" => unimplemented!(),
            b"1.05" => unimplemented!(),
            b"1.06" => unimplemented!(),
            b"1.07" => Self::load_121(format_version, 1.07, input),
            b"1.08" => unimplemented!(),
            b"1.09" | b"1.10" | b"1.11" => Self::load_121(format_version, 1.11, input),
            b"1.12" | b"1.13" | b"1.14" | b"1.15" | b"1.16" => Self::load_121(format_version, 1.12, input),
            b"1.17" => Self::load_121(format_version, 1.14, input),
            b"1.18" | b"1.19" => Self::load_121(format_version, 1.13, input),
            b"1.20" | b"1.21" => Self::load_121(format_version, 1.14, input),
            // Definitive Edition
            b"3.13" => Self::load_121(format_version, 1.14, input),
            _ => Err(Error::new(ErrorKind::Other, format!("Unsupported format version {:?}", format_version))),
        }
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: &VersionBundle) -> Result<()> {
        let player_version = match &version.format {
            b"1.07" => 1.07,
            b"1.09" | b"1.10" | b"1.11" => 1.11,
            b"1.12" | b"1.13" | b"1.14" | b"1.15" | b"1.16" => 1.12,
            b"1.18" | b"1.19" => 1.13,
            b"1.14" | b"1.20" | b"1.21" => 1.14,
            _ => panic!("writing version {} is not supported", String::from_utf8_lossy(&version.format)),
        };

        output.write_all(&version.format)?;
        self.header.write_to(output, version.format, version.header)?;

        let mut output = DeflateEncoder::new(output, Compression::default());
        output.write_i32::<LE>(self.next_object_id)?;

        self.tribe_scen.write_to(&mut output, version.data)?;
        self.map.write_to(&mut output)?;

        output.write_i32::<LE>(self.player_objects.len() as i32)?;
        for player in &self.world_players {
            player.write_to(&mut output, player_version)?;
        }

        for objects in &self.player_objects {
            output.write_i32::<LE>(objects.len() as i32)?;
            for object in objects {
                object.write_to(&mut output, version.format)?;
            }
        }

        output.write_i32::<LE>(self.scenario_players.len() as i32 + 1)?;
        for player in &self.scenario_players {
            player.write_to(&mut output, player_version, version.victory)?;
        }

        if cmp_scx_version(version.format, *b"1.13") == Ordering::Greater {
            let def = TriggerSystem::default();
            let triggers = match self.triggers {
                Some(ref tr) => tr,
                None => &def,
            };
            triggers.write_to(&mut output, version.triggers)?;
        }

        if cmp_scx_version(version.format, *b"1.17") == Ordering::Greater && cmp_scx_version(version.format, *b"2.00") == Ordering::Less {
            let def = AIInfo::default();
            let ai_info = match self.ai_info {
                Some(ref ai) => ai,
                None => &def,
            };
            ai_info.write_to(&mut output)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SCXFormat;
    use crate::VersionBundle;
    use std::fs::File;

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=42
    #[test]
    fn oldest_aoe1_scn_on_aoeheaven() {
        let mut f = File::open("test/scenarios/ The Destruction of Rome.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &format.version()).expect("failed to write");
    }

    #[test]
    fn aoe1_beta_scn_reserialize() {
        let mut f = File::open("test/scenarios/Dawn of a New Age.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &format.version()).expect("failed to write");

        let mut f = std::io::Cursor::new(out);
        let format2 = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_eq!(format!("{:#?}", format), format!("{:#?}", format2), "should produce exactly the same scenario");
    }

    #[test]
    fn aoe1_beta_scn_to_aoc() {
        let mut f = File::open("test/scenarios/Dawn of a New Age.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &VersionBundle::aoc()).expect("failed to write");

        let mut f = std::io::Cursor::new(out);
        let format2 = SCXFormat::load_scenario(&mut f).expect("failed to read");
        assert_eq!(format2.version(), VersionBundle::aoc(), "should have converted to AoC versions");
    }

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=1678
    #[test]
    fn aoe1_trial_scn() {
        let mut f = File::open("test/scenarios/Bronze Age Art of War.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &format.version()).expect("failed to write");
    }

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=2409
    #[test]
    fn aoe1_ppc_trial_scn() {
        let mut f = File::open("test/scenarios/CEASAR.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &format.version()).expect("failed to write");
    }

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=1651
    #[test]
    fn aoe1_scn() {
        let mut f = File::open("test/scenarios/A New Emporer.scn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &format.version()).expect("failed to write");
    }

    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=880
    #[test]
    fn aoe1_ror_scx() {
        let mut f = File::open("test/scenarios/Jeremiah Johnson (Update).scx").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &format.version()).expect("failed to write");
    }

    /// Source: http://aok.heavengames.com/blacksmith/showfile.php?fileid=1271
    #[test]
    fn oldest_aok_scn_on_aokheaven() {
        let mut f = File::open("test/scenarios/CAMELOT.SCN").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &format.version()).expect("failed to write");
    }

    #[test]
    fn aoc_scx() {
        let mut f = File::open("test/scenarios/Age of Heroes b1-3-5.scx").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &format.version()).expect("failed to write");
    }

    #[test]
    fn hd_aoe2scenario() {
        let mut f = File::open("test/scenarios/Year_of_the_Pig.aoe2scenario").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &format.version()).expect("failed to write");
    }

    #[test]
    fn hd_scx2() {
        let mut f = File::open("test/scenarios/real_world_amazon.scx").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &format.version()).expect("failed to write");
    }

    /// A Definitive Edition scenario.
    ///
    /// (Ignored because it doesn't work yet.)
    /// Source: http://aoe.heavengames.com/dl-php/showfile.php?fileid=2708
    #[test]
    #[ignore]
    fn aoe_de_scn() {
        let mut f = File::open("test/scenarios/Corlis.aoescn").unwrap();
        let format = SCXFormat::load_scenario(&mut f).expect("failed to read");
        let mut out = vec![];
        format.write_to(&mut out, &format.version()).expect("failed to write");
    }
}
