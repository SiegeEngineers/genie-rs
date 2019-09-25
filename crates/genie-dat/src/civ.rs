use crate::{
    unit_type::{UnitType, UnitTypeID},
    GameVersion,
};
use arrayvec::ArrayString;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{fallible_try_from, fallible_try_into, infallible_try_into};
use std::{
    convert::TryInto,
    io::{Read, Result, Write},
};
use encoding_rs::WINDOWS_1252;

/// An ID identifying a civilization
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CivilizationID(u16);

impl From<u16> for CivilizationID {
    fn from(n: u16) -> Self {
        CivilizationID(n)
    }
}

impl From<CivilizationID> for u16 {
    fn from(n: CivilizationID) -> Self {
        n.0
    }
}

impl From<CivilizationID> for usize {
    fn from(n: CivilizationID) -> Self {
        n.0.into()
    }
}

fallible_try_into!(CivilizationID, i16);
infallible_try_into!(CivilizationID, u32);
fallible_try_from!(CivilizationID, i32);
fallible_try_from!(CivilizationID, u32);

type CivName = ArrayString<[u8; 20]>;

#[derive(Debug, Default, Clone)]
pub struct Civilization {
    name: CivName,
    attributes: Vec<f32>,
    civ_effect: u16,
    bonus_effect: Option<u16>,
    culture: u8,
    unit_types: Vec<Option<UnitType>>,
}

impl Civilization {
    /// Get the name of this civilization.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn from<R: Read>(input: &mut R, version: GameVersion) -> Result<Self> {
        let mut civ = Self::default();
        let mut bytes = [0; 20];
        input.read_exact(&mut bytes)?;
        let bytes = &bytes[..bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len())];
        let (name, _encoding, _failed) = WINDOWS_1252.decode(&bytes);
        civ.name = CivName::from(&name).unwrap();
        let num_attributes = input.read_u16::<LE>()?;
        civ.civ_effect = input.read_u16::<LE>()?;
        civ.bonus_effect = {
            let id = input.read_u16::<LE>()?;
            if id == 0xFFFF {
                None
            } else {
                Some(id)
            }
        };

        for _ in 0..num_attributes {
            civ.attributes.push(input.read_f32::<LE>()?);
        }

        civ.culture = input.read_u8()?;

        let num_unit_types = input.read_u16::<LE>()?;
        let have_unit_types = {
            let mut list = vec![];
            for _ in 0..num_unit_types {
                list.push(input.read_u32::<LE>()? != 0);
            }
            list
        };
        for do_read in have_unit_types {
            if !do_read {
                civ.unit_types.push(None);
                continue;
            }
            civ.unit_types.push(Some(UnitType::from(input, version)?));
        }

        Ok(civ)
    }

    pub fn write_to<W: Write>(&self, output: &mut W, version: GameVersion) -> Result<()> {
        let mut name = [0; 20];
        (&mut name[..]).copy_from_slice(self.name.as_bytes());
        output.write_all(&name)?;
        output.write_u16::<LE>(self.attributes.len().try_into().unwrap())?;
        output.write_u16::<LE>(self.civ_effect)?;
        output.write_u16::<LE>(self.bonus_effect.unwrap_or(0xFFFF))?;
        for v in self.attributes.iter() {
            output.write_f32::<LE>(*v)?;
        }
        output.write_u8(self.culture)?;

        output.write_u16::<LE>(self.unit_types.len().try_into().unwrap())?;
        for opt in &self.unit_types {
            output.write_u32::<LE>(match opt {
                Some(_) => 1,
                None => 0,
            })?;
        }
        for opt in &self.unit_types {
            if let Some(unit_type) = opt {
                unit_type.write_to(output, version)?;
            }
        }
        Ok(())
    }

    /// Get a unit type by its ID.
    pub fn get_unit_type(&self, id: impl Into<UnitTypeID>) -> Option<&UnitType> {
        let id: UnitTypeID = id.into();
        self.unit_types
            .get(usize::from(id))
            .and_then(Option::as_ref)
    }
}
