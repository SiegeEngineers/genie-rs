use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use genie_support::{fallible_try_from, infallible_try_into};
use std::{
    convert::TryInto,
    io::{Read, Result, Write},
};

/// A palette index.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PaletteIndex(u8);
impl From<u8> for PaletteIndex {
    fn from(n: u8) -> Self {
        PaletteIndex(n)
    }
}

impl From<PaletteIndex> for u8 {
    fn from(n: PaletteIndex) -> Self {
        n.0
    }
}

impl From<PaletteIndex> for usize {
    fn from(n: PaletteIndex) -> Self {
        n.0.into()
    }
}

fallible_try_from!(PaletteIndex, i32);
fallible_try_from!(PaletteIndex, u32);
infallible_try_into!(PaletteIndex, i16);
infallible_try_into!(PaletteIndex, i32);
infallible_try_into!(PaletteIndex, u32);

/// Player colour data.
#[derive(Debug, Clone)]
pub struct ColorTable {
    id: i32,
    /// Base palette index for this player colour.
    base: PaletteIndex,
    /// The palette index to use for unit outlines when they are obscured by buildings or trees.
    unit_outline_color: PaletteIndex,
    unit_selection_colors: (PaletteIndex, PaletteIndex),
    /// Palette indices for this colour on the minimap.
    minimap_colors: (PaletteIndex, PaletteIndex, PaletteIndex),
    /// Color table to use for this player colour in the in-game statistics in the bottom right.
    statistics_text_color: i32,
}

impl ColorTable {
    pub fn from<R: Read>(input: &mut R) -> Result<Self> {
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
