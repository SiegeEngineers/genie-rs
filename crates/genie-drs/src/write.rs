use crate::{DRSHeader, DRSResource, DRSTable, ResourceType};
use byteorder::{WriteBytesExt, LE};
use std::io::{Read, Result, Seek, SeekFrom, Write};

/// Strategy to use when writing files to the archive.
///
/// DRS files contain metadata at the start of the archive, and the size of this metadata depends
/// on the number of files, the different types of files, and the sizes of the files that are added
/// to the archive. To correctly write an archive to disk, we need to know the number and sizes of
/// files before writing the header, and we only know where to put the files once the size of the
/// header is known. In practice, this means we have to keep the entire archive in memory before
/// writing it.
///
/// There are tricks to work around this and reduce memory usage. The `InMemory` strategy keeps the
/// file in memory before writing it and is great for small archives. The other strateg(y|ies) start writing files without needing to keep them entirely in memory, with other tradeoffs.
pub enum Strategy {
    /// Create the entire DRS file in memory first, then flush it to the output.
    ///
    /// This works best for archives with small files.
    InMemory,
    /// Reserve space for metadata for the given amount of tables and files at the top of the file, then fill
    /// it in at the end.
    ///
    /// The first number is the amount of tables, the second is the amount of files.
    ReserveDirectory(u32, u32),
}

impl Default for Strategy {
    /// The default strategy reserves space for 65536 files spread across 10 tables, meaning a directory size of about 768KiB.
    fn default() -> Self {
        Strategy::ReserveDirectory(10, 65536)
    }
}

/// Generator for .drs archives.
///
/// ```rust
/// use std::{io::Cursor, fs::File};
/// use genie_drs::{DRSWriter, WriteStrategy};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let buf = Cursor::new(vec![]);
/// let mut writer = DRSWriter::new(buf, WriteStrategy::InMemory)?;
/// writer.add("bina", 50500, "JASC-PAL\r\n0100\r\n...".as_bytes())?;
/// writer.add("slp", 2, &b"some bytes"[..])?;
/// let buf = writer.flush()?;
/// // → a Vec<u8> containing the DRS file
/// # Ok(()) }
/// ```
pub struct DRSWriter<W>
where
    W: Write + Seek,
{
    inner: W,
    header: DRSHeader,
    tables: Vec<DRSTable>,
    resources: Vec<Vec<u8>>,
    write_offset: u32,
    strategy: Strategy,
}

impl<W> DRSWriter<W>
where
    W: Write + Seek,
{
    /// Create a writer with the given strategy.
    pub fn new(output: W, strategy: Strategy) -> Result<Self> {
        let header = DRSHeader {
            banner_msg: *b"Copyright (c) 1997 Ensemble Studios.\x1a\x00\x00\x00",
            version: *b"1.00",
            password: *b"tribe\x00\x00\x00\x00\x00\x00\x00",
            num_resource_types: 0,
            directory_size: 0,
        };

        let mut writer = Self {
            inner: output,
            header,
            tables: vec![],
            resources: if let Strategy::InMemory = strategy {
                vec![]
            } else {
                Vec::with_capacity(0)
            },
            write_offset: 0,
            strategy,
        };

        writer.write_reserved()?;

        Ok(writer)
    }

    /// Write 0 bytes for reserved space at the top of the file.
    fn write_reserved(&mut self) -> Result<()> {
        if let Strategy::ReserveDirectory(tables, files) = self.strategy {
            self.header.directory_size = 64 + 12 * (tables + files);
            self.write_header()?;
            let reserved = vec![0; self.header.directory_size as usize - 64];
            self.inner.write_all(&reserved)?;
            self.write_offset = self.header.directory_size;
        }
        Ok(())
    }

    /// Write the .drs archive header.
    fn write_header(&mut self) -> Result<()> {
        self.header.write_to(&mut self.inner)
    }

    /// Write the table and resource data.
    fn write_tables(&mut self) -> Result<()> {
        for table in &self.tables {
            table.write_to(&mut self.inner)?;
        }
        for table in &self.tables {
            for resource in table.resources() {
                resource.write_to(&mut self.inner)?;
            }
        }
        Ok(())
    }

    /// Add a file to the archive.
    #[inline]
    pub fn add(&mut self, t: impl Into<ResourceType>, id: u32, data: impl Read) -> Result<()> {
        self.add_inner(t.into(), id, data)
    }

    fn add_inner(&mut self, t: ResourceType, id: u32, mut data: impl Read) -> Result<()> {
        let res = DRSResource {
            id,
            offset: 0, // TBD
            size: 0,   // TBD
        };

        let res = match self
            .tables
            .iter_mut()
            .find(|table| table.resource_type == t)
        {
            Some(table) => table.add(res),
            None => {
                let mut table = DRSTable {
                    resource_type: t,
                    offset: 0, // TBD
                    num_resources: 0,
                    resources: vec![],
                };
                table.add(res);
                self.tables.push(table);
                self.tables
                    .last_mut()
                    .unwrap()
                    .resources
                    .last_mut()
                    .unwrap()
            }
        };

        match self.strategy {
            Strategy::InMemory => {
                let mut bytes = vec![];
                data.read_to_end(&mut bytes)?;
                assert!(bytes.len() < u32::max_value() as usize, "file too large");
                res.size = bytes.len() as u32;
                self.resources.push(bytes);
            }
            Strategy::ReserveDirectory(_, _) => {
                let len = std::io::copy(&mut data, &mut self.inner)?;
                assert!(len < u32::max_value() as u64, "file too large");
                res.offset = self.write_offset;
                res.size = len as u32;
                self.write_offset += res.size;
            }
        }

        Ok(())
    }

    /// Finish any writes that still need to happen and return the file handle.
    pub fn flush(mut self) -> Result<W> {
        assert!(
            self.tables.len() < u32::max_value() as usize,
            "too many tables"
        );
        self.header.num_resource_types = self.tables.len() as u32;

        match self.strategy {
            Strategy::InMemory => {
                assert!(
                    self.resources.len() < u32::max_value() as usize,
                    "too many resources"
                );
                self.header.directory_size = 64 + 12 * (self.tables.len() + self.resources.len()) as u32;
                unimplemented!();
            }
            Strategy::ReserveDirectory(_, _) => {
                // Update the resource type count
                self.inner.seek(SeekFrom::Start(56))?;
                self.inner.write_u32::<LE>(self.header.num_resource_types)?;
                self.inner.seek(SeekFrom::Current(4))?;

                // Assign table offsets
                let mut table_offset = 64 + 12 * (self.tables.len() as u32);
                for table in self.tables.iter_mut() {
                    table.offset = table_offset;
                    table_offset += 12 * (table.len() as u32);
                }

                // Write out all the table data
                self.write_tables()?;
                self.inner.seek(SeekFrom::End(0))?;
            }
        }

        Ok(self.inner)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;

    #[test]
    fn basic() {
        let f = File::create("/tmp/x.drs").unwrap();
        let mut drs = DRSWriter::new(f, Strategy::ReserveDirectory(1, 1)).unwrap();
        drs.add("txt", 1, "example test file".as_bytes()).unwrap();
        drs.flush().unwrap();
    }
}
