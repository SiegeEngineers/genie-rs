use byteorder::{ReadBytesExt, LE};
use std::convert::{TryFrom, TryInto};
use std::io::{self, Error, ErrorKind, Read, Result};

/// Read a 2-byte integer that uses -1 as an "absent" value.
///
/// ## Example
///
/// ```rust
/// use genie_support::read_opt_u16;
///
/// let mut minus_one = std::io::Cursor::new(vec![0xFF, 0xFF]);
/// let mut zero = std::io::Cursor::new(vec![0x00, 0x00]);
///
/// assert_eq!(read_opt_u16(&mut minus_one).unwrap(), None);
/// assert_eq!(read_opt_u16(&mut zero).unwrap(), Some(0));
/// ```
#[inline]
pub fn read_opt_u16<R: Read>(input: &mut R) -> Result<Option<u16>> {
    let v = input.read_i16::<LE>()?;
    if v == -1 {
        return Ok(None);
    }
    Ok(Some(
        v.try_into()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?,
    ))
}

/// Read a 4-byte integer that uses -1 as an "absent" value.
///
/// ## Example
///
/// ```rust
/// use genie_support::read_opt_u32;
///
/// let mut minus_one = std::io::Cursor::new(vec![0xFF, 0xFF, 0xFF, 0xFF]);
/// let mut one = std::io::Cursor::new(vec![0x01, 0x00, 0x00, 0x00]);
///
/// assert_eq!(read_opt_u32::<u32>(&mut minus_one).unwrap(), None);
/// assert_eq!(read_opt_u32(&mut one).unwrap(), Some(1));
/// ```
#[inline]
pub fn read_opt_u32<T>(input: &mut impl Read) -> Result<Option<T>>
where
    T: TryFrom<u32>,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    let opt = match input.read_u32::<LE>()? {
        0xFFFF_FFFF => None,
        v => Some(
            v.try_into()
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))?,
        ),
    };
    Ok(opt)
}

/// Extension trait that adds a `skip()` method to `Read` instances.
pub trait ReadSkipExt {
    /// Read and discard a number of bytes.
    fn skip(&mut self, dist: u64) -> Result<()>;
}

impl<T: Read> ReadSkipExt for T {
    fn skip(&mut self, dist: u64) -> Result<()> {
        io::copy(&mut self.by_ref().take(dist), &mut io::sink())?;
        Ok(())
    }
}
