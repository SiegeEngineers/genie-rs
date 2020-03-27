use byteorder::{WriteBytesExt, LE};
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

/// Read a string of length `length` from an input stream, using code page 1252.
pub fn read_str<R: Read>(input: &mut R, length: usize) -> Result<Option<String>, ReadStringError> {
    if length > 0 {
        let mut bytes = vec![0; length as usize];
        input.read_exact(&mut bytes)?;
        if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
            bytes.truncate(end);
        }
        if bytes.is_empty() {
            Ok(None)
        } else {
            let (result, _enc, failed) = WINDOWS_1252.decode(&bytes);
            if failed {
                Err(ReadStringError::DecodeStringError(DecodeStringError))
            } else {
                Ok(Some(result.to_string()))
            }
        }
    } else {
        Ok(None)
    }
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
        write_str(output, &string)
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
        write_i32_str(output, &string)
    } else {
        output.write_i32::<LE>(0)?;
        Ok(())
    }
}
