use crate::unit_type::UnitType;
use arrayvec::ArrayString;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::{
    convert::TryInto,
    io::{Read, Result, Write},
};

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
    pub fn from<R: Read>(input: &mut R, _player_type: i8) -> Result<Self> {
        let mut civ = Self::default();
        let mut bytes = [0; 20];
        input.read_exact(&mut bytes)?;
        bytes
            .iter()
            .cloned()
            .take_while(|b| *b != 0)
            .map(char::from)
            .for_each(|c| civ.name.push(c));
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
            civ.unit_types.push(Some(UnitType::from(input)?));
        }

        Ok(civ)
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        let mut name = [0; 20];
        (&mut name[..]).copy_from_slice(self.name.as_bytes());
        output.write_all(&name)?;
        output.write_u16::<LE>(self.attributes.len().try_into().unwrap())?;
        output.write_u16::<LE>(self.civ_effect)?;
        output.write_u16::<LE>(self.bonus_effect.unwrap_or(0xFFFF))?;
        output.write_u8(self.culture)?;
        for v in self.attributes.iter() {
            output.write_f32::<LE>(*v)?;
        }
        Ok(())
    }
}
