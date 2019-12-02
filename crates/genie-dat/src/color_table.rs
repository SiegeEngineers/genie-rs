use byteorder::{ReadBytesExt, WriteBytesExt, LE};
pub use jascpal::PaletteIndex;
use std::{
    convert::TryInto,
    io::{Read, Result, Write},
};

/// Player colour data.
#[derive(Debug, Clone)]
pub struct ColorTable {
    pub id: i32,
    /// Base palette index for this player colour.
    pub base: PaletteIndex,
    /// The palette index to use for unit outlines when they are obscured by buildings or trees.
    pub unit_outline_color: PaletteIndex,
    pub unit_selection_colors: (PaletteIndex, PaletteIndex),
    /// Palette indices for this colour on the minimap.
    pub minimap_colors: (PaletteIndex, PaletteIndex, PaletteIndex),
    /// Color table to use for this player colour in the in-game statistics in the bottom right.
    pub statistics_text_color: i32,
}

impl ColorTable {
    pub fn read_from<R: Read>(input: &mut R) -> Result<Self> {
        let id = input.read_i32::<LE>()?;
        let base = input.read_i32::<LE>()?.try_into().unwrap();
        let unit_outline_color = input.read_i32::<LE>()?.try_into().unwrap();
        let unit_selection_colors = (
            input.read_i32::<LE>()?.try_into().unwrap(),
            input.read_i32::<LE>()?.try_into().unwrap(),
        );
        let minimap_colors = (
            input.read_i32::<LE>()?.try_into().unwrap(),
            input.read_i32::<LE>()?.try_into().unwrap(),
            input.read_i32::<LE>()?.try_into().unwrap(),
        );
        let statistics_text_color = input.read_i32::<LE>()?;

        Ok(Self {
            id,
            base,
            unit_outline_color,
            unit_selection_colors,
            minimap_colors,
            statistics_text_color,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<LE>(self.id)?;
        output.write_i32::<LE>(self.base.try_into().unwrap())?;
        output.write_i32::<LE>(self.unit_outline_color.try_into().unwrap())?;
        output.write_i32::<LE>(self.unit_selection_colors.0.try_into().unwrap())?;
        output.write_i32::<LE>(self.unit_selection_colors.1.try_into().unwrap())?;
        output.write_i32::<LE>(self.minimap_colors.0.try_into().unwrap())?;
        output.write_i32::<LE>(self.minimap_colors.1.try_into().unwrap())?;
        output.write_i32::<LE>(self.minimap_colors.2.try_into().unwrap())?;
        output.write_i32::<LE>(self.statistics_text_color)?;
        Ok(())
    }
}
