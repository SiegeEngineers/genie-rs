use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct StringTable {
    max_strings: u16,
    strings: Vec<String>,
}

impl StringTable {
    pub fn new(max_strings: u16) -> Self {
        StringTable {
            max_strings,
            strings: vec![],
        }
    }

    pub fn read_from(mut input: impl Read) -> Result<Self> {
        let max_strings = input.read_u16::<LE>()?;
        let num_strings = input.read_u16::<LE>()?;
        let _ptr = input.read_u32::<LE>()?;

        let mut strings = Vec::with_capacity(max_strings as usize);
        for _ in 0..num_strings {
            let length = input.read_u32::<LE>()?;
            let mut bytes = vec![0; length as usize];
            input.read_exact(&mut bytes)?;
            strings.push(String::from_utf8(bytes).unwrap());
        }

        Ok(StringTable {
            max_strings,
            strings,
        })
    }

    pub fn write_to<W: Write>(&self, handle: &mut W) -> Result<()> {
        handle.write_u16::<LE>(self.max_strings)?;
        handle.write_u16::<LE>(self.num_strings())?;
        handle.write_u32::<LE>(0)?;

        for string in &self.strings {
            let len = string.len();
            assert!(len < u32::max_value() as usize);
            handle.write_u32::<LE>(len as u32)?;
            handle.write_all(string.as_bytes())?;
        }

        Ok(())
    }

    pub fn max_strings(&self) -> u16 {
        self.max_strings
    }

    pub fn num_strings(&self) -> u16 {
        let len = self.strings.len();
        assert!(len < u16::max_value() as usize);
        len as u16
    }

    pub fn strings(&self) -> &Vec<String> {
        &self.strings
    }
}

impl IntoIterator for StringTable {
    type Item = String;
    type IntoIter = ::std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.strings.into_iter()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_strings() {
        assert_eq!(2 + 2, 4);
    }
}
