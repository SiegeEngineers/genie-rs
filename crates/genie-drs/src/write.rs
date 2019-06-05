use crate::{DRSHeader, DRSResource, DRSTable, ResourceType};
use byteorder::{WriteBytesExt, LE};
use std::io::{Read, Result, Seek, SeekFrom, Write};

/// Strategy to use when writing files to the archive.
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
    /// Reserves space for 65536 files spread across 10 tables, meaning a directory size of about 768KiB.
    fn default() -> Self {
        Strategy::ReserveDirectory(10, 65536)
    }
}

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

    pub fn into_inner(self) -> W {
        self.inner
    }

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

    fn write_header(&mut self) -> Result<()> {
        self.header.write_to(&mut self.inner)
    }

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

    pub fn add(&mut self, t: ResourceType, id: u32, mut data: impl Read) -> Result<()> {
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
                assert!(bytes.len() < u32::max_value() as usize);
                res.size = bytes.len() as u32;
                self.resources.push(bytes);
            }
            Strategy::ReserveDirectory(_, _) => {
                let len = std::io::copy(&mut data, &mut self.inner)?;
                assert!(len < u32::max_value() as u64);
                res.offset = self.write_offset;
                res.size = len as u32;
                self.write_offset += res.size;
            }
        }

        Ok(())
    }

    pub fn flush(mut self) -> Result<W> {
        if let Strategy::ReserveDirectory(_, _) = self.strategy {
            // Update the resource type count
            assert!(self.tables.len() < u32::max_value() as usize);
            let num_resource_types = self.tables.len() as u32;
            self.inner.seek(SeekFrom::Start(56))?;
            self.inner.write_u32::<LE>(num_resource_types)?;
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
        drs.add(*b" txt", 1, "example test file".as_bytes())
            .unwrap();
        drs.flush().unwrap();
    }
}
