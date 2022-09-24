use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use encoding_rs::WINDOWS_1252;
use std::io::{self, Read, Write};

/// Failed to decode a string as WINDOWS-1252.
///
/// This means that the scenario file contained a string that could not be decoded using the
/// WINDOWS-1252 code page. In the future, genie-scx will support other encodings.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("could not decode string as WINDOWS-1252")]
pub struct DecodeStringError;

/// Failed to encode a string as WINDOWS-1252.
///
/// This means that a string could not be encoded using the WINDOWS-1252 code page. In the future, genie-scx will support other encodings.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("could not encode string as WINDOWS-1252")]
pub struct EncodeStringError;

/// Failed to read a string.
#[derive(Debug, thiserror::Error)]
pub enum ReadStringError {
    /// Failed to read a string because the bytes could not be decoded.
    #[error(transparent)]
    DecodeStringError(#[from] DecodeStringError),
    /// Failed to read a string because the underlying I/O failed.
    #[error(transparent)]
    IoError(#[from] io::Error),
}

/// Failed to write a string.
#[derive(Debug, thiserror::Error)]
pub enum WriteStringError {
    /// Failed to read a string because it could not be encoded.
    #[error(transparent)]
    EncodeStringError(#[from] EncodeStringError),
    /// Failed to write a string because the underlying I/O failed.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

/// Write a string to an output stream, using code page 1252, using a `u16` for the length prefix.
///
/// This writes the length of the string (including NULL terminator) as a little-endian u16,
/// followed by the encoded bytes, followed by a NULL terminator.
pub fn write_str<W: Write>(output: &mut W, string: &str) -> Result<(), WriteStringError> {
    let (bytes, _enc, failed) = WINDOWS_1252.encode(string);
    if failed {
        return Err(WriteStringError::EncodeStringError(EncodeStringError));
    }
    assert!(bytes.len() < std::i16::MAX as usize);
    output.write_i16::<LE>(bytes.len() as i16 + 1)?;
    output.write_all(&bytes)?;
    output.write_u8(0)?;
    Ok(())
}

/// Write a string to an output stream, using code page 1252, using a `u32` for the length prefix.
///
/// This writes the length of the string (including NULL terminator) as a little-endian u177,
/// followed by the encoded bytes, followed by a NULL terminator.
pub fn write_i32_str<W: Write>(output: &mut W, string: &str) -> Result<(), WriteStringError> {
    let (bytes, _enc, failed) = WINDOWS_1252.encode(string);
    if failed {
        return Err(WriteStringError::EncodeStringError(EncodeStringError));
    }
    assert!(bytes.len() < std::i32::MAX as usize);
    output.write_i32::<LE>(bytes.len() as i32 + 1)?;
    output.write_all(&bytes)?;
    output.write_u8(0)?;
    Ok(())
}

/// Write a string to an output stream, using code page 1252, using a `u16` for the length prefix.
///
/// When given a `None`, it outputs a 0 for the length. Otherwise, see `write_str`.
pub fn write_opt_str<W: Write>(
    output: &mut W,
    option: &Option<String>,
) -> Result<(), WriteStringError> {
    if let Some(ref string) = option {
        write_str(output, string)
    } else {
        output.write_i16::<LE>(0)?;
        Ok(())
    }
}

/// Write a string to an output stream, using code page 1252, using a `u32` for the length prefix.
///
/// When given a `None`, it outputs a 0 for the length. Otherwise, see `write_str`.
pub fn write_opt_i32_str<W: Write>(
    output: &mut W,
    option: &Option<String>,
) -> Result<(), WriteStringError> {
    if let Some(ref string) = option {
        write_i32_str(output, string)
    } else {
        output.write_i32::<LE>(0)?;
        Ok(())
    }
}

/// Decode a string using the WINDOWS-1252 code page.
fn decode_str(bytes: &[u8]) -> Result<String, DecodeStringError> {
    if bytes.is_empty() {
        return Ok("".to_string());
    }

    let (decoded, _enc, failed) = WINDOWS_1252.decode(bytes);
    if failed {
        Err(DecodeStringError)
    } else {
        Ok(decoded.to_string())
    }
}

/// Functions to read various kinds of strings from input streams.
/// Extension trait for reading strings in several common formats used by AoE2.
pub trait ReadStringsExt: Read {
    /// Read an optionally null-terminated WINDOWS-1252-encoded string with the given `length` in bytes.
    fn read_str(&mut self, length: usize) -> Result<Option<String>, ReadStringError> {
        if length > 0 {
            let mut bytes = vec![0; length as usize];
            self.read_exact(&mut bytes)?;
            if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
                bytes.truncate(end);
            }
            if bytes.is_empty() {
                Ok(None)
            } else {
                Ok(Some(decode_str(&bytes)?))
            }
        } else {
            Ok(None)
        }
    }

    /// Read an u16 value, then read an optionally null-terminated WINDOWS-1252-encoded string of
    /// that length in bytes.
    fn read_u16_length_prefixed_str(&mut self) -> Result<Option<String>, ReadStringError> {
        match self.read_u16::<LE>()? {
            0xFFFF => Ok(None),
            len => self.read_str(len as usize),
        }
    }

    /// Read an u32 value, then read an optionally null-terminated WINDOWS-1252-encoded string of
    /// that length in bytes.
    fn read_u32_length_prefixed_str(&mut self) -> Result<Option<String>, ReadStringError> {
        match self.read_u32::<LE>()? {
            0xFFFF_FFFF => Ok(None),
            len => self.read_str(len as usize),
        }
    }

    /// Read an HD Edition style string.
    ///
    /// Reads a 'signature' value, then the `length` as an u16 value, then reads an optionally
    /// null-terminated WINDOWS-1252-encoded string of that length in bytes.
    fn read_hd_style_str(&mut self) -> Result<Option<String>, ReadStringError> {
        let open = self.read_u16::<LE>()?;
        // Check that this actually is the start of a string
        if open != 0x0A60 {
            return Err(DecodeStringError.into());
        }
        let len = self.read_u16::<LE>()? as usize;
        let mut bytes = vec![0; len];
        self.read_exact(&mut bytes[0..len])?;
        Ok(Some(decode_str(&bytes)?))
    }
}

impl<T> ReadStringsExt for T where T: Read {}
