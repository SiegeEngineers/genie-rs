//! Parser for the Age of Empires 1/2 graphic file format, SLP.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused)]

use byteorder::{ReadBytesExt, LE};
pub use jascpal::PaletteIndex;
pub use rgb::RGBA8;
use std::ffi::CStr;
use std::fmt::{self, Debug, Display};
use std::io::{Cursor, Read, Result, Seek, SeekFrom};

/// Trait for graphic formats.
pub trait Format: Sized {
    /// The data type for a single pixel.
    type Pixel: Copy;

    /// Read a draw command from an input stream.
    fn read_command<R: Read>(reader: R) -> Result<Command<Self::Pixel>>;
}

/// Trait for SLP formats. SLP formats differ in the size of the pixel data.
pub trait SLPFormat {
    /// The data type for a single pixel.
    type Pixel: Copy;

    /// Read a single pixel value from an input stream.
    fn read_pixel<R: Read>(reader: R) -> Result<Self::Pixel>;
}

/// The classic 8-bit palette-based pixel format, used across all versions.
pub struct PalettePixelFormat;
/// The 32-bit RGBA pixel format introduced in Age of Empires 2: HD Edition.
pub struct RGBAPixelFormat;

impl SLPFormat for PalettePixelFormat {
    type Pixel = PaletteIndex;

    fn read_pixel<R: Read>(mut reader: R) -> Result<Self::Pixel> {
        reader.read_u8().map(Into::into)
    }
}

impl SLPFormat for RGBAPixelFormat {
    type Pixel = RGBA8;

    fn read_pixel<R: Read>(mut reader: R) -> Result<Self::Pixel> {
        let [r, g, b, a] = reader.read_u32::<LE>()?.to_le_bytes();
        Ok(RGBA8 { r, g, b, a })
    }
}

impl<S: SLPFormat> Format for S {
    type Pixel = S::Pixel;

    fn read_command<R: Read>(mut reader: R) -> Result<Command<Self::Pixel>> {
        fn read_pixels<R, S>(num_pixels: u32, mut reader: R) -> Result<Vec<S::Pixel>>
        where
            R: Read,
            S: SLPFormat,
        {
            let mut pixels = Vec::with_capacity(num_pixels as usize);
            for _ in 0..num_pixels {
                pixels.push(S::read_pixel(&mut reader)?);
            }
            Ok(pixels)
        }
        let command = reader.read_u8()?;
        if command & 0b1111 == 0b1111 {
            return Ok(Command::NextLine);
        }
        if command & 0b11 == 0b00 {
            let num_pixels = u32::from(command >> 2);
            let pixels = read_pixels::<&mut R, S>(num_pixels, &mut reader)?;
            return Ok(Command::Copy(pixels));
        }
        if command & 0b11 == 0b01 {
            let num_pixels = u32::from(if command >> 2 != 0 {
                command >> 2
            } else {
                reader.read_u8()?
            });
            return Ok(Command::Skip(num_pixels));
        }
        if command & 0b1111 == 0b0010 {
            let num_pixels = u32::from(((command & 0b1111_0000) << 4) + reader.read_u8()?);
            let pixels = read_pixels::<&mut R, S>(num_pixels, &mut reader)?;
            return Ok(Command::Copy(pixels));
        }
        if command & 0b1111 == 0b0011 {
            let num_pixels = u32::from(((command & 0b1111_0000) << 4) + reader.read_u8()?);
            return Ok(Command::Skip(num_pixels));
        }
        if command & 0b1111 == 0b0110 {
            let num_pixels = u32::from(if command >> 4 != 0 {
                command >> 4
            } else {
                reader.read_u8()?
            });
            let pixels = read_pixels::<&mut R, S>(num_pixels, &mut reader)?;
            return Ok(Command::PlayerCopy(pixels));
        }
        if command & 0b1111 == 0b0111 {
            let num_pixels = u32::from(if command >> 4 != 0 {
                command >> 4
            } else {
                reader.read_u8()?
            });
            let pixel = S::read_pixel(&mut reader)?;
            return Ok(Command::Fill(num_pixels, pixel));
        }
        if command & 0b1111 == 0b1010 {
            let num_pixels = u32::from(if command >> 4 != 0 {
                command >> 4
            } else {
                reader.read_u8()?
            });
            let pixel = S::read_pixel(&mut reader)?;
            return Ok(Command::PlayerFill(num_pixels, pixel));
        }
        unimplemented!()
    }
}

/// An SLP command.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command<Pixel: Copy> {
    /// Copy pixels to the output.
    Copy(Vec<Pixel>),
    /// Copy pixels to the output, applying a player colour transformation.
    PlayerCopy(Vec<Pixel>),
    /// Fill this many pixels in the output with a specific colour.
    Fill(u32, Pixel),
    /// Fill this many pixels in the output with a specific colour, applying a player colour
    /// transformation.
    PlayerFill(u32, Pixel),
    /// Skip this many pixels in the output.
    Skip(u32),
    /// Continue to the next line.
    NextLine,
}

impl<Pixel: Copy> Command<Pixel> {
    /// Transform the pixel colour values in this command.
    ///
    /// # Examples
    /// Use `map_color()` to apply a palette:
    ///
    /// ```rust
    /// use genie_slp::{Command, RGBA8};
    ///
    /// let mut palette = vec![RGBA8::default(); 256];
    /// palette[63] = RGBA8::new(0, 0xFF, 0x00, 0xFF); // green
    /// palette[127] = RGBA8::new(0, 0, 0xFF, 0xFF); // blue
    /// let command = Command::Copy(vec![127, 127, 63, 63]);
    /// let command = command.map_color(|palette_index: u8| palette[palette_index as usize]);
    /// assert_eq!(
    ///     command,
    ///     Command::Copy(vec![
    ///         RGBA8::new(0, 0, 0xFF, 0xFF),
    ///         RGBA8::new(0, 0, 0xFF, 0xFF),
    ///         RGBA8::new(0, 0xFF, 0, 0xFF),
    ///         RGBA8::new(0, 0xFF, 0, 0xFF),
    ///     ])
    /// );
    /// ```
    pub fn map_color<OutputPixel: Copy>(
        self,
        mut transform: impl FnMut(Pixel) -> OutputPixel,
    ) -> Command<OutputPixel> {
        match self {
            Self::Copy(pixels) => Command::Copy(pixels.into_iter().map(transform).collect()),
            Self::Fill(num, pixel) => Command::Fill(num, transform(pixel)),
            Self::PlayerCopy(_pixels) => todo!(),
            Self::PlayerFill(_num, _pixel) => todo!(),
            Self::Skip(num) => Command::Skip(num),
            Self::NextLine => Command::NextLine,
        }
    }

    /*
        pub fn to_writer<W: Write>(&self, w: &mut W) -> Result<()> {
            match self {
                Command::Copy(colors) => {
                    let n = colors.len();
                    if n >= 64 {
                        w.write_u8(0x02 | ((n & 0xF00) >> 4))?;
                        w.write_u8(n & 0xFF)?;
                    } else {
                        w.write_u8(0x00 | (n << 2))?;
                    }
                    w.write_all(&colors)?;
                }
                _ => unimplemented!(),
            }
            Ok(())
        }

        pub fn to_bytes(&self) -> Vec<u8> {
            let mut v = vec![];
            self.to_writer(&mut v).unwrap();
            v
        }
    */
}

/// SLP file version identifier.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SLPVersion([u8; 4]);

impl Display for SLPVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::str::from_utf8(&self.0).unwrap())
    }
}

impl Debug for SLPVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SLPVersion({})", self)
    }
}

/// Outline data: how many pixels at the start and end of each row should be transparent.
#[derive(Debug, Default)]
pub struct Outline {
    /// How many pixels should be transparent at the start of the row.
    pub left: u16,
    /// How many pixels should be transparent at the end of the row.
    pub right: u16,
}

impl Outline {
    /// Read SLP frame row outline data from an input stream.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let left = input.read_u16::<LE>()?;
        let right = input.read_u16::<LE>()?;

        Ok(Self { left, right })
    }

    /// Is this entire row transparent?
    pub fn is_transparent(&self) -> bool {
        ((self.left | self.right) & 0x8000) == 0x8000
    }
}

#[derive(Debug, Default)]
struct SLPFrameMeta {
    command_table_offset: u32,
    outline_table_offset: u32,
    palette_offset: u32,
    properties: u32,
    width: u32,
    height: u32,
    hotspot: (i32, i32),
    outlines: Vec<Outline>,
    command_offsets: Vec<usize>,
}

impl SLPFrameMeta {
    pub(crate) fn read_from(mut input: impl Read) -> Result<Self> {
        let mut frame = Self::default();
        frame.command_table_offset = input.read_u32::<LE>()?;
        frame.outline_table_offset = input.read_u32::<LE>()?;
        frame.palette_offset = input.read_u32::<LE>()?;
        frame.properties = input.read_u32::<LE>()?;
        frame.width = input.read_u32::<LE>()?;
        frame.height = input.read_u32::<LE>()?;
        frame.hotspot = (input.read_i32::<LE>()?, input.read_i32::<LE>()?);
        Ok(frame)
    }

    pub(crate) fn read_outlines(&mut self, input: &[u8]) -> Result<()> {
        let mut input = Cursor::new(&input[(self.outline_table_offset as usize)..]);
        for _ in 0..self.height {
            self.outlines.push(Outline::read_from(&mut input)?);
        }
        Ok(())
    }

    pub(crate) fn read_command_offsets(&mut self, input: &[u8]) -> Result<()> {
        let mut input = Cursor::new(&input[(self.command_table_offset as usize)..]);
        for _ in 0..self.height {
            self.command_offsets.push(input.read_u32::<LE>()? as usize);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SLPFrame<'slp> {
    meta: &'slp SLPFrameMeta,
    buffer: &'slp [u8],
}

impl<'slp> SLPFrame<'slp> {
    fn new(meta: &'slp SLPFrameMeta, buffer: &'slp [u8]) -> Self {
        Self { meta, buffer }
    }

    /// Get the size of this frame in pixels. Returns `(width, height)`.
    pub fn size(&self) -> (u32, u32) {
        (self.meta.width, self.meta.height)
    }

    /// Get the hotspot location of this frame.
    pub fn hotspot(&self) -> (i32, i32) {
        self.meta.hotspot
    }

    /// Does this frame contain 32 bit pixel data?
    pub fn is_32bit(&self) -> bool {
        self.meta.properties & 7 == 7
    }

    /// Does this frame contain 8 bit pixel data?
    pub fn is_8bit(&self) -> bool {
        !self.is_32bit()
    }

    /// Iterate over the commands in this 8 bit frame.
    ///
    /// # Panics
    /// This function panics if this frame's pixel format is not 8 bit.
    pub fn render_8bit(&self) -> SLPFrameCommands<'_, PalettePixelFormat> {
        assert!(self.is_8bit(), "render_8bit() called on a 32 bit frame");
        SLPFrameCommands::new(self.buffer, &self.meta.command_offsets)
    }

    /// Iterate over the commands in this 32 bit frame.
    ///
    /// # Panics
    /// This function panics if this frame's pixel format is not 32 bit.
    pub fn render_32bit(&self) -> SLPFrameCommands<'_, RGBAPixelFormat> {
        assert!(self.is_32bit(), "render_32bit() called on an 8 bit frame");
        SLPFrameCommands::new(self.buffer, &self.meta.command_offsets)
    }
}

#[derive(Debug)]
pub struct SLP {
    bytes: Vec<u8>,
    version: SLPVersion,
    comment: String,
    frames: Vec<SLPFrameMeta>,
}

impl SLP {
    /// Read an SLP file from an input stream. This reads the full stream into memory.
    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let mut bytes = Vec::new();
        input.read_to_end(&mut bytes)?;
        Self::from_bytes(bytes)
    }

    /// Read an SLP file from a byte array.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let mut input = Cursor::new(&bytes);
        let version = {
            let mut bytes = [0; 4];
            input.read_exact(&mut bytes)?;
            SLPVersion(bytes)
        };
        let num_frames = input.read_i32::<LE>()? as u32 as usize;
        let comment = {
            let mut bytes = [0; 24];
            input.read_exact(&mut bytes)?;
            CStr::from_bytes_with_nul(&bytes)
                .expect("could not create CStr from comment")
                .to_str()
                .expect("comment not utf-8")
                .to_string()
        };

        let mut frames = Vec::with_capacity(num_frames);
        for _ in 0..num_frames {
            frames.push(SLPFrameMeta::read_from(&mut input)?);
        }

        for frame in frames.iter_mut() {
            frame.read_outlines(&bytes)?;
            frame.read_command_offsets(&bytes)?;
        }

        Ok(Self {
            bytes,
            version,
            comment,
            frames,
        })
    }

    /// Iterate over the frames in this SLP file.
    pub fn frames(&self) -> impl Iterator<Item = SLPFrame<'_>> + '_ {
        self.frames
            .iter()
            .map(move |frame| SLPFrame::new(frame, &self.bytes))
    }

    /// Get an individual frame.
    ///
    /// # Panics
    /// This function panics if the `index` is out of bounds.
    pub fn frame(&self, index: usize) -> SLPFrame<'_> {
        SLPFrame::new(&self.frames[index], &self.bytes)
    }

    /// Get the number of frames in this SLP file.
    pub fn num_frames(&self) -> usize {
        self.frames.len()
    }
}

/// Iterator over commands in an SLP frame.
pub struct SLPFrameCommands<'a, F>
where
    F: Format,
{
    line: u32,
    end: bool,
    offsets: &'a [usize],
    buffer: Cursor<&'a [u8]>,
    _format: std::marker::PhantomData<F>,
}

impl<'a, F> SLPFrameCommands<'a, F>
where
    F: Format,
{
    fn new(bytes: &'a [u8], offsets: &'a [usize]) -> Self {
        let mut buffer = Cursor::new(bytes);
        buffer.seek(SeekFrom::Start(offsets[0] as u64)).unwrap();
        Self {
            _format: std::marker::PhantomData,
            line: 0,
            end: false,
            offsets,
            buffer,
        }
    }

    /// Jump to the next line. Remaining commands on the current line will not be read.
    pub fn next_line(&mut self) {
        self.line += 1;
        match self.offsets.get(self.line as usize).copied() {
            Some(offset) => {
                self.buffer.seek(SeekFrom::Start(offset as u64)).unwrap();
            }
            None => {
                // Return `None` the next time `next()` is called.
                self.end = true;
            }
        }
    }
}

impl<'a, F> Iterator for SLPFrameCommands<'a, F>
where
    F: Format,
{
    type Item = Result<Command<F::Pixel>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        match F::read_command(&mut self.buffer) {
            Ok(command) => match command {
                Command::NextLine => {
                    self.next_line();
                    Some(Ok(command))
                }
                command => Some(Ok(command)),
            },
            err @ Err(_) => Some(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn parse_slp() -> anyhow::Result<()> {
        let f = File::open("test/fixtures/eslogo1.slp")?;
        let slp = SLP::read_from(f)?;
        assert_eq!(slp.num_frames(), 2);
        assert!(slp.frame(0).is_32bit());
        assert!(slp.frame(1).is_32bit());
        assert!(!slp.frame(0).is_8bit());
        assert!(!slp.frame(1).is_8bit());
        assert_eq!(slp.frame(0).size(), (127, 92));
        assert_eq!(slp.frame(1).size(), (127, 92));
        Ok(())
    }

    #[test]
    fn render_32bit() -> anyhow::Result<()> {
        let f = File::open("test/fixtures/eslogo1.slp")?;
        let slp = SLP::read_from(f)?;
        for command in slp.frame(0).render_32bit() {
            let _ = command?;
        }
        Ok(())
    }

    #[test]
    #[should_panic = "render_8bit() called on a 32 bit frame"]
    fn render_32bit_slp_as_8bit() {
        let f = File::open("test/fixtures/eslogo1.slp").unwrap();
        let slp = SLP::read_from(f).unwrap();
        for _ in slp.frame(0).render_8bit() {}
    }
}
