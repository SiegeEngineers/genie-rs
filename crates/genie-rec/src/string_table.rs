use std::io::{Read, Write, Error};
use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};

pub struct StringTable {
    max_strings: u16,
    strings: Vec<String>,
}

impl StringTable {
    pub fn new(max_strings: u16) -> Self {
        StringTable { max_strings, strings: vec![] }
    }

    pub fn from<R: Read>(handle: &mut R) -> Result<Self, Error> {
        let max_strings = handle.read_u16::<LE>()?;
        let num_strings = handle.read_u16::<LE>()?;
        let _ptr = handle.read_u32::<LE>()?; // unsure why this is here

        let mut strings = Vec::with_capacity(max_strings as usize);
        for _ in 0..num_strings {
            let length = handle.read_u32::<LE>()?;
            let mut string = String::with_capacity(length as usize);
            unsafe {
                handle.take(length as u64)
                    .read_to_end(&mut string.as_mut_vec())?;
            }
            strings.push(string);
        }

        Ok(StringTable { max_strings, strings })
    }

    pub fn write_to<W: Write>(&self, handle: &mut W) -> Result<(), Error> {
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
    use std::fs::File;

    #[test]
    fn read_strings() {
        assert_eq!(2 + 2, 4);
    }
}
