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
/// assert_eq!(read_opt_u16::<u16, _>(&mut minus_one).unwrap(), None);
/// assert_eq!(read_opt_u16(&mut zero).unwrap(), Some(0));
/// ```
#[inline]
pub fn read_opt_u16<T, R>(mut input: R) -> Result<Option<T>>
where
    T: TryFrom<u16>,
    T::Error: std::error::Error + Send + Sync + 'static,
    R: Read,
{
    let opt = match input.read_u16::<LE>()? {
        0xFFFF => None,
        v => Some(
            v.try_into()
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))?,
        ),
    };
    Ok(opt)
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
/// assert_eq!(read_opt_u32::<u32, _>(&mut minus_one).unwrap(), None);
/// assert_eq!(read_opt_u32(&mut one).unwrap(), Some(1));
/// ```
#[inline]
pub fn read_opt_u32<T, R>(mut input: R) -> Result<Option<T>>
where
    T: TryFrom<u32>,
    T::Error: std::error::Error + Send + Sync + 'static,
    R: Read,
{
    let opt = match input.read_u32::<LE>()? {
        0xFFFF_FFFF => None,
        // HD Edition uses -2 in some places.
        0xFFFF_FFFE => None,
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

/// Very simple struct that tracks the position inside a `Read`
pub struct Tracker<T> {
    inner: T,
    position: u64,
}

impl<T> Tracker<T> {
    pub fn new(inner: T) -> Self {
        Tracker { inner, position: 0 }
    }

    pub fn position(&self) -> u64 {
        self.position
    }
}

impl<T: Read> Read for Tracker<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read = self.inner.read(buf)?;
        self.position += read as u64;
        Ok(read)
    }
}
