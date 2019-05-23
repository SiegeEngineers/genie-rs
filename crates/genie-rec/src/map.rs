use std::io::{Read, Result};
use byteorder::{ReadBytesExt, LE};

pub struct Tile {
    terrain: u8,
    elevation: u8,
    original_terrain: Option<u8>,
}

impl Tile {
    fn from<R: Read>(input: &mut R) -> Result<Self> {
        let terrain = input.read_u8()?;
        let (terrain, elevation, original_terrain) = if terrain == 0xFF {
            (input.read_u8()?, input.read_u8()?, Some(input.read_u8()?))
        } else {
            (terrain, input.read_u8()?, None)
        };
        Ok(Tile { terrain, elevation, original_terrain })
    }

    fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        match self.original_terrain {
            Some(t) => {
                output.write_u8(0xFF)?;
                output.write_u8(self.terrain)?;
                output.write_u8(self.elevation)?;
                output.write_u8(t)?;
            },
            None => {
                output.write_u8(self.terrain)?;
                output.write_u8(self.elevation)?;
            },
        }
        Ok(())
    }
}
