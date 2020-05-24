use crate::types::{DLCPackage, DataSet, SCXVersion};
use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{read_str, write_opt_i32_str};
use std::convert::TryFrom;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct DLCOptions {
    /// Version of the DLC options structure.
    pub version: i32,
    /// The base data set.
    pub game_data_set: DataSet,
    /// The DLCs that are required by this scenario.
    pub dependencies: Vec<DLCPackage>,
}

impl Default for DLCOptions {
    fn default() -> Self {
        Self {
            version: 1000,
            game_data_set: DataSet::BaseGame,
            dependencies: vec![DLCPackage::AgeOfKings, DLCPackage::AgeOfConquerors],
        }
    }
}

impl DLCOptions {
    /// Read DLC options from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        // If version is 0 or 1, it's actually the dataset identifier from
        // before DLCOptions was versioned.
        let version_or_data_set = input.read_i32::<LE>()?;
        let game_data_set =
            DataSet::try_from(if version_or_data_set == 0 || version_or_data_set == 1 {
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
        let mut dependencies = vec![DLCPackage::AgeOfKings; num_dependencies as usize];
        for dependency in dependencies.iter_mut() {
            *dependency = DLCPackage::try_from(input.read_i32::<LE>()?)?;
        }

        Ok(DLCOptions {
            version,
            game_data_set,
            dependencies,
        })
    }

    /// Write DLC options to an output stream.
    pub fn write_to(&self, mut output: impl Write) -> Result<()> {
        output.write_u32::<LE>(1000)?;
        output.write_i32::<LE>(self.game_data_set.into())?;
        output.write_u32::<LE>(self.dependencies.len() as u32)?;
        for dlc_id in &self.dependencies {
            output.write_i32::<LE>((*dlc_id).into())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
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
    /// Name of the person who created this scenario. Only available in DE2.
    pub author_name: Option<String>,
    /// Whether the scenario has any victory conditions for singleplayer.
    pub any_sp_victory: bool,
    /// How many players are supported by this scenario.
    pub active_player_count: u32,
    /// HD Edition DLC information.
    pub dlc_options: Option<DLCOptions>,
}

impl SCXHeader {
    /// Parse an SCX header from a byte stream.
    pub fn read_from(mut input: impl Read, format_version: SCXVersion) -> Result<SCXHeader> {
        let _header_size = input.read_u32::<LE>()?;
        let version = input.read_u32::<LE>()?;
        log::debug!("Header version {}", version);
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
        let description = read_str(&mut input, description_length)?;

        let any_sp_victory = input.read_u32::<LE>()? != 0;
        let active_player_count = input.read_u32::<LE>()?;

        let dlc_options = if version > 2 && format_version != *b"3.13" {
            Some(DLCOptions::read_from(&mut input)?)
        } else {
            None
        };

        let author_name;
        if version >= 5 {
            author_name = {
                let len = input.read_u32::<LE>()?;
                read_str(&mut input, len as usize)?
            };
            let _num_triggers = input.read_u32::<LE>()?;
        } else {
            author_name = None;
        }

        Ok(SCXHeader {
            version,
            timestamp,
            description,
            author_name,
            any_sp_victory,
            active_player_count,
            dlc_options,
        })
    }

    /// Serialize an SCX header to a byte stream.
    pub fn write_to(
        &self,
        output: impl Write,
        format_version: SCXVersion,
        version: u32,
    ) -> Result<()> {
        let mut intermediate = vec![];

        intermediate.write_u32::<LE>(version)?;

        if version >= 2 {
            intermediate.write_u32::<LE>(self.timestamp)?;
        }

        let mut description_bytes = vec![];
        if let Some(ref description) = self.description {
            description_bytes.write_all(description.as_bytes())?;
        }
        description_bytes.push(0);
        if format_version == *b"3.13" {
            assert!(
                description_bytes.len() <= std::u16::MAX as usize,
                "description length must fit in u16"
            );
            intermediate.write_u16::<LE>(description_bytes.len() as u16)?;
        } else {
            assert!(
                description_bytes.len() <= std::u32::MAX as usize,
                "description length must fit in u32"
            );
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

        if version >= 5 {
            write_opt_i32_str(&mut intermediate, &self.author_name)?;
            // TODO should be number of triggers
            intermediate.write_u32::<LE>(0)?;
        }

        // Make `output` mutable here so we don't accidentally use it above.
        let mut output = output;
        output.write_u32::<LE>(intermediate.len() as u32)?;
        output.write_all(&intermediate)?;

        Ok(())
    }

    /// Update the timestamp.
    pub fn touch(&mut self) -> std::result::Result<(), std::time::SystemTimeError> {
        let system_time = std::time::SystemTime::now();
        let duration = system_time.duration_since(std::time::UNIX_EPOCH)?;
        self.timestamp = duration.as_secs() as u32;
        Ok(())
    }
}
