//! JASC Palette file reader/writer.
//!
//! Parses and stringifes palette files containing any number of colours.
//!
//! JASC palette files follow a simple format:
//! ```txt
//! JASC-PAL
//! 0100
//! $num_colors
//! $rgb...
//! ```
//!
//! Colours are represented using the [`rgb`](https://crates.io/crates/rgb) crate.
//!
//! ## Example
//! ```rust
//! use std::io::Cursor;
//! use jascpal::{Palette, PaletteIndex, Color};
//! let cursor = Cursor::new(b"JASC-PAL\r\n0100\r\n2\r\n0 0 0\r\n255 255 255\r\n".to_vec());
//! let pal = Palette::read_from(cursor).unwrap();
//! assert_eq!(pal[PaletteIndex::from(0)], Color { r: 0, g: 0, b: 0 });
//! assert_eq!(pal[PaletteIndex::from(1)], Color { r: 255, g: 255, b: 255 });
//! let mut pal = pal;
//! pal[PaletteIndex::from(1)] = Color { r: 0, g: 255, b: 255 };
//! pal.add(Color { r: 255, g: 0, b: 0 });
//! assert_eq!(
//!     pal.to_string(),
//!     "JASC-PAL\r\n0100\r\n3\r\n0 0 0\r\n0 255 255\r\n255 0 0\r\n".to_string()
//! );
//! ```
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, one_of},
    combinator::{map, map_res},
    multi::many1,
    IResult,
};
use rgb::RGB;
use std::{
    convert::{TryFrom, TryInto},
    num::TryFromIntError,
    fmt,
    io::{Read, Write},
    str::{self, FromStr},
};

/// Represents an RGB colour.
pub type Color = RGB<u8>;

/// A palette index.
///
/// ## Example
/// ```rust
/// use jascpal::{Palette, PaletteIndex, Color};
/// let pal = Palette::default();
/// assert_eq!(pal[PaletteIndex::from(0)], Color { r: 0, g: 0, b: 0 });
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PaletteIndex(u8);
impl From<u8> for PaletteIndex {
    #[inline]
    fn from(n: u8) -> Self {
        PaletteIndex(n)
    }
}

impl TryFrom<i32> for PaletteIndex {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(n: i32) -> Result<Self, Self::Error> {
        n.try_into().map(Self)
    }
}

impl From<PaletteIndex> for u8 {
    #[inline]
    fn from(n: PaletteIndex) -> Self {
        n.0
    }
}

impl From<PaletteIndex> for i32 {
    #[inline]
    fn from(n: PaletteIndex) -> Self {
        n.0.into()
    }
}

impl From<PaletteIndex> for usize {
    #[inline]
    fn from(n: PaletteIndex) -> Self {
        n.0.into()
    }
}

impl FromStr for PaletteIndex {
    type Err = std::num::ParseIntError;
    #[inline]
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        input.parse().map(Self)
    }
}

/// Eat any amount of whitespace: ASCII spaces and tabs.
fn whitespace(input: &[u8]) -> IResult<&[u8], ()> {
    map(many1(one_of(&b" \t"[..])), |_| ())(input)
}

/// Parse an UTF-8 byte slice using FromStr.
fn parse_bytes<Parsed: FromStr>(string: &[u8]) -> Result<Parsed, ()> {
    str::from_utf8(string)
        .map_err(|_| ())
        .and_then(|s| s.parse().map_err(|_| ()))
}

/// Eat and return a number.
fn parse_number<Parsed: FromStr>(input: &[u8]) -> IResult<&[u8], Parsed> {
    map_res(digit1, parse_bytes)(input)
}

/// Eat and return an RGB value (three `u8` components).
fn parse_rgb(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, r) = parse_number(input)?;
    let (input, _) = whitespace(input)?;
    let (input, g) = parse_number(input)?;
    let (input, _) = whitespace(input)?;
    let (input, b) = parse_number(input)?;

    Ok((input, RGB { r, g, b }))
}

/// Parse a header and colours from an input stream.
fn parse(input: &[u8]) -> IResult<&[u8], Vec<Color>> {
    let (input, _) = tag(b"JASC-PAL\r\n")(input)?;
    let (input, _) = tag(b"0100\r\n")(input)?;
    let (input, num_colors) = parse_number(input)?;
    let (input, _) = tag(b"\r\n")(input)?;

    let mut colors = Vec::with_capacity(num_colors as usize);
    let mut input = input;
    for _ in 0..num_colors {
        let (remaining, color) = parse_rgb(input)?;
        colors.push(color);
        let (remaining, _) = tag(b"\r\n")(remaining)?;
        input = remaining;
    }

    Ok((input, colors))
}

/// An error occurred during reading.
#[derive(Debug)]
pub enum ReadPaletteError {
    /// An I/O error occurred.
    IoError(std::io::Error),
    /// The palette could not be parsed.
    ParseError,
}

impl fmt::Display for ReadPaletteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ReadPaletteError::*;
        match self {
            IoError(err) => write!(f, "{}", err),
            ParseError => write!(f, "parse error"),
        }
    }
}

impl std::error::Error for ReadPaletteError {}

impl From<std::io::Error> for ReadPaletteError {
    fn from(err: std::io::Error) -> Self {
        ReadPaletteError::IoError(err)
    }
}

/// A Palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    colors: Vec<Color>,
}

impl From<Vec<Color>> for Palette {
    /// Create a palette from a vector of colours.
    #[inline]
    fn from(colors: Vec<Color>) -> Self {
        Palette { colors }
    }
}

impl Default for Palette {
    /// Create a palette with 256 colours, all black.
    #[inline]
    fn default() -> Self {
        Self::from(vec![Default::default(); 256])
    }
}

impl Palette {
    /// Create an empty palette.
    #[inline]
    pub fn new() -> Self {
        Self::from(vec![])
    }

    /// Read a palette from an input stream.
    #[inline]
    pub fn read_from(mut input: impl Read) -> Result<Self, ReadPaletteError> {
        let mut buffer = vec![];
        input.read_to_end(&mut buffer)?;
        let (_remaining, pal) = parse(&buffer).map_err(|_| ReadPaletteError::ParseError)?;
        Ok(Self::from(pal))
    }

    /// Write a palette to an output stream.
    #[inline]
    pub fn write_to(&self, mut output: impl Write) -> Result<(), std::io::Error> {
        output.write_all(b"JASC-PAL\r\n0100\r\n")?;
        output.write_all(format!("{}\r\n", self.colors.len()).as_bytes())?;
        for c in &self.colors {
            output.write_all(format!("{} {} {}\r\n", c.r, c.g, c.b).as_bytes())?;
        }
        Ok(())
    }

    /// Return the colours in a Vec so it can be manipulated.
    ///
    /// It can later be wrapped back into a Palette instance using `Palette::from(vec)`.
    #[inline]
    pub fn into_inner(self) -> Vec<Color> {
        self.colors
    }

    /// Return the colours as a slice.
    #[inline]
    pub fn colors(&self) -> &[Color] {
        &self.colors
    }

    /// Return the colours as a mutable vector.
    #[inline]
    pub fn colors_mut(&mut self) -> &mut Vec<Color> {
        &mut self.colors
    }

    /// Returns the number of colours in this palette.
    #[inline]
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    /// Returns `true` if this palette contains 0 colours.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// Add a colour at the end of the palette.
    #[inline]
    pub fn add(&mut self, color: Color) {
        self.colors.push(color);
    }

    /// Serialize the palette to a byte vector.
    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        self.write_to(&mut bytes).unwrap();
        bytes
    }
}

impl std::ops::Index<PaletteIndex> for Palette {
    type Output = Color;
    /// Get the colour at the given index.
    #[inline]
    fn index(&self, index: PaletteIndex) -> &Self::Output {
        let index: usize = index.into();
        &self.colors[index]
    }
}

impl std::ops::IndexMut<PaletteIndex> for Palette {
    /// Get the colour at the given index.
    #[inline]
    fn index_mut(&mut self, index: PaletteIndex) -> &mut Self::Output {
        let index: usize = index.into();
        &mut self.colors[index]
    }
}

impl std::str::FromStr for Palette {
    type Err = ReadPaletteError;
    /// Parse a palette from a UTF-8 string.
    #[inline]
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::read_from(input.as_bytes())
    }
}

impl IntoIterator for Palette {
    type Item = Color;
    type IntoIter = std::vec::IntoIter<Color>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.colors.into_iter()
    }
}

impl<'a> IntoIterator for &'a Palette {
    type Item = &'a Color;
    type IntoIter = std::slice::Iter<'a, Color>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.colors.iter()
    }
}

impl ToString for Palette {
    #[inline]
    fn to_string(&self) -> String {
        let s = self.to_bytes();
        unsafe { String::from_utf8_unchecked(s) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_test() {
        parse(b"JASC-PAL\r\n0100\r\n2\r\n0 0 0\r\n255 255 255\r\n").unwrap();
    }

    #[test]
    fn from_str() {
        assert_eq!(
            "JASC-PAL\r\n0100\r\n3\r\n255 0 255\r\n0 0 255\r\n0 255 0\r\n"
                .parse::<Palette>()
                .unwrap(),
            Palette::from(vec![
                RGB {
                    r: 255,
                    g: 0,
                    b: 255
                },
                RGB { r: 0, g: 0, b: 255 },
                RGB { r: 0, g: 255, b: 0 },
            ])
        );
    }

    #[test]
    fn make() {
        let mut pal = Palette::new();
        for r in 0..=255 {
            pal.add(RGB {
                r,
                g: 255 - r,
                b: 127_u8.wrapping_add(r),
            });
        }
        let s = pal.to_string();
        let mut lines = s.lines();
        assert_eq!(lines.next(), Some("JASC-PAL"));
        assert_eq!(lines.next(), Some("0100"));
        assert_eq!(lines.next(), Some("256"));
        for r in 0..=255 {
            assert_eq!(
                lines.next(),
                Some(format!("{} {} {}", r, 255 - r, 127_u8.wrapping_add(r)).as_str())
            );
        }
        assert_eq!(lines.next(), None);
    }

    #[test]
    fn it_works() {
        let cursor = Cursor::new(b"JASC-PAL\r\n0100\r\n2\r\n0 0 0\r\n255 255 255\r\n".to_vec());
        let pal = Palette::read_from(cursor).unwrap();
        assert_eq!(
            pal.colors,
            vec![
                RGB { r: 0, g: 0, b: 0 },
                RGB {
                    r: 255,
                    g: 255,
                    b: 255
                },
            ]
        );
        assert_eq!(
            pal.to_string(),
            "JASC-PAL\r\n0100\r\n2\r\n0 0 0\r\n255 255 255\r\n".to_string()
        );
    }
}
