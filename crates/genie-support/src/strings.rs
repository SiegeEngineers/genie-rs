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
