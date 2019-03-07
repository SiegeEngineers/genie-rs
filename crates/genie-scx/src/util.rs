use std::io::{
    Read,
    Write,
    Result,
    Error,
    ErrorKind,
};
use byteorder::{WriteBytesExt, LE};
use encoding_rs::WINDOWS_1252;

pub fn read_str<R: Read>(input: &mut R, length: usize) -> Result<Option<String>> {
    if length > 0 {
        let mut bytes = vec![0; length as usize];
        input.read_exact(&mut bytes)?;
        if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
            bytes.truncate(end);
        }
        if bytes.is_empty() {
            Ok(None)
        } else {
            let (result, enc, failed) = WINDOWS_1252.decode(&bytes);
            if failed {
                Err(Error::new(ErrorKind::Other, "invalid string"))
            } else {
                Ok(Some(result.to_string()))
            }
        }
    } else {
        Ok(None)
    }
}

pub fn write_str<W: Write>(output: &mut W, string: &str) -> Result<()> {
    let bytes = string.as_bytes();
    assert!(bytes.len() < std::i16::MAX as usize);
    output.write_i16::<LE>(bytes.len() as i16 + 1)?;
    output.write_all(bytes)?;
    output.write_u8(0)?;
    Ok(())
}

pub fn write_i32_str<W: Write>(output: &mut W, string: &str) -> Result<()> {
    let bytes = string.as_bytes();
    assert!(bytes.len() < std::i32::MAX as usize);
    output.write_i32::<LE>(bytes.len() as i32 + 1)?;
    output.write_all(bytes)?;
    output.write_u8(0)?;
    Ok(())
}

pub fn write_opt_str<W: Write>(output: &mut W, option: &Option<String>) -> Result<()> {
    if let Some(ref string) = option {
        write_str(output, &string)
    } else {
        output.write_i16::<LE>(0)?;
        Ok(())
    }
}

pub fn write_opt_i32_str<W: Write>(output: &mut W, option: &Option<String>) -> Result<()> {
    if let Some(ref string) = option {
        write_i32_str(output, &string)
    } else {
        output.write_i32::<LE>(0)?;
        Ok(())
    }
}
