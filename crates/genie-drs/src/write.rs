use crate::{DRSHeader, DRSResource, DRSTable, ResourceType};
use byteorder::{WriteBytesExt, LE};
use std::io::{self, Read, Seek, SeekFrom, Write};

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
pub trait Strategy<W>
where
    W: Write + Seek,
{
    fn open(&mut self, drs: &mut InnerDRSWriter<W>) -> Result<(), io::Error>;
    fn add_resource(
        &mut self,
        drs: &mut InnerDRSWriter<W>,
        table: ResourceType,
        resource: DRSResource,
        data: &mut dyn Read,
    ) -> Result<DRSResource, io::Error>;
    fn close(&mut self, drs: &mut InnerDRSWriter<W>) -> Result<(), io::Error>;
}

/// Create the entire DRS file in memory first, then flush it to the output.
///
/// This works best for archives with small files.
#[derive(Default)]
pub struct InMemoryStrategy {
    resources: Vec<(ResourceType, Vec<u8>)>,
}

impl<W> Strategy<W> for InMemoryStrategy
where
    W: Write + Seek,
{
    fn open(&mut self, _drs: &mut InnerDRSWriter<W>) -> Result<(), io::Error> {
        Ok(())
    }

    fn add_resource(
        &mut self,
        _drs: &mut InnerDRSWriter<W>,
        table: ResourceType,
        mut resource: DRSResource,
        data: &mut dyn Read,
    ) -> Result<DRSResource, io::Error> {
        let mut bytes = vec![];
        data.read_to_end(&mut bytes)?;
        assert!(bytes.len() < u32::max_value() as usize, "file too large");
        resource.size = bytes.len() as u32;
        self.resources.push((table, bytes));
        Ok(resource)
    }

    fn close(&mut self, drs: &mut InnerDRSWriter<W>) -> Result<(), io::Error> {
        assert!(
            self.resources.len() < u32::max_value() as usize,
            "too many resources"
        );

        let num_tables = drs.tables.len();
        let num_resources = drs.tables.iter().fold(0, |acc, t| acc + t.len());
        drs.header.directory_size = 64 + 12 * (num_tables + num_resources) as u32;
        drs.write_header()?;

        // Assign table offsets
        let mut table_offset = 64 + 12 * (drs.tables.len() as u32);
        let mut file_offset = drs.header.directory_size;
        for table in drs.tables.iter_mut() {
            table.offset = table_offset;
            table_offset += 12 * (table.len() as u32);

            // Assign file offsets
            for res in table.resources.iter_mut() {
                res.offset = file_offset;
                file_offset += res.size;
            }
        }

        // Write out all the table data
        drs.write_tables()?;

        for table in &drs.tables {
            let mut data = self.resources.iter().filter_map(|(t, bytes)| {
                if table.resource_type == *t {
                    Some(bytes)
                } else {
                    None
                }
            });
            for _ in &table.resources {
                let bytes = data.next().expect("genie-drs bug: mismatch between InMemoryStrategy resources and DRSWriter table data");
                drs.output.write_all(&bytes)?;
            }
        }

        Ok(())
    }
}

/// Writer strategy that reserve space for metadata for the given amount of tables and files at the top of the file, then fills it in at the end.
pub struct ReserveDirectoryStrategy {
    reserved_tables: u32,
    file_space_left: u32,
    write_offset: u32,
}

impl ReserveDirectoryStrategy {
    /// Create a write strategy using a reserved metadata block.
    ///
    /// The strategy will support up to `reserved_tables` tables and `reserved_files` files.
    /// The total size of the reserved metadata block will be `12 * (reserved_tables +
    /// reserved_files)` bytes.
    #[inline]
    pub fn new(reserved_tables: u32, reserved_files: u32) -> Self {
        Self {
            reserved_tables,
            file_space_left: reserved_files,
            write_offset: 64 + 12 * (reserved_tables + reserved_files),
        }
    }
}

impl<W> Strategy<W> for ReserveDirectoryStrategy
where
    W: Write + Seek,
{
    fn open(&mut self, drs: &mut InnerDRSWriter<W>) -> Result<(), io::Error> {
        drs.header.directory_size = self.write_offset;
        drs.write_header()?;

        // Write 0 bytes for reserved space at the top of the file.
        let reserved_block = vec![0; self.write_offset as usize - 64];
        drs.output.write_all(&reserved_block)?;
        Ok(())
    }

    fn add_resource(
        &mut self,
        drs: &mut InnerDRSWriter<W>,
        _table: ResourceType,
        mut resource: DRSResource,
        data: &mut dyn Read,
    ) -> Result<DRSResource, io::Error> {
        assert!(self.file_space_left > 0, "too many files");
        self.file_space_left -= 1;

        let len = std::io::copy(data, &mut drs.output)?;
        assert!(len < u64::from(u32::max_value()), "file too large");
        resource.offset = self.write_offset;
        resource.size = len as u32;
        self.write_offset += resource.size;
        Ok(resource)
    }

    fn close(&mut self, drs: &mut InnerDRSWriter<W>) -> Result<(), io::Error> {
        assert!(
            drs.tables.len() <= self.reserved_tables as usize,
            "too many tables"
        );

        // Update the resource type count
        drs.output.seek(SeekFrom::Start(56))?;
        drs.output.write_u32::<LE>(drs.header.num_resource_types)?;
        drs.output.seek(SeekFrom::Current(4))?;

        // Assign table offsets
        let mut table_offset = 64 + 12 * (drs.tables.len() as u32);
        for table in drs.tables.iter_mut() {
            table.offset = table_offset;
            table_offset += 12 * (table.len() as u32);
        }

        // Write out all the table data
        drs.write_tables()?;
        drs.output.seek(SeekFrom::End(0))?;

        Ok(())
    }
}

impl Default for ReserveDirectoryStrategy {
    /// The default strategy reserves space for 65536 files spread across 10 tables, meaning a directory size of about 768KiB.
    #[inline]
    fn default() -> Self {
        Self::new(10, 65536)
    }
}

pub struct InnerDRSWriter<W>
where
    W: Write + Seek,
{
    output: W,
    header: DRSHeader,
    tables: Vec<DRSTable>,
}

impl<W> InnerDRSWriter<W>
where
    W: Write + Seek,
{
    /// Write the .drs archive header.
    fn write_header(&mut self) -> io::Result<()> {
        self.header.write_to(&mut self.output)
    }

    /// Write the table and resource data.
    fn write_tables(&mut self) -> io::Result<()> {
        for table in &self.tables {
            table.write_to(&mut self.output)?;
        }
        for table in &self.tables {
            for resource in table.resources() {
                resource.write_to(&mut self.output)?;
            }
        }
        Ok(())
    }

    /// Consume the writer and return the underlying Write instance.
    fn into_inner(self) -> W {
        self.output
    }
}

/// Generator for .drs archives.
///
/// ```rust
/// use std::{io::Cursor, fs::File};
/// use genie_drs::{DRSWriter, InMemoryStrategy};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let buf = Cursor::new(vec![]);
/// let mut writer = DRSWriter::new(buf, InMemoryStrategy::default())?;
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
    inner: InnerDRSWriter<W>,
    strategy: Box<dyn Strategy<W>>,
}

impl<W> DRSWriter<W>
where
    W: Write + Seek,
{
    /// Create a writer with the given strategy.
    pub fn new(output: W, strategy: impl Strategy<W> + 'static) -> io::Result<Self> {
        let header = DRSHeader::default();

        let mut writer = Self {
            inner: InnerDRSWriter {
                output,
                header,
                tables: vec![],
            },
            strategy: Box::new(strategy),
        };

        writer.strategy.open(&mut writer.inner)?;

        Ok(writer)
    }

    /// Add a file to the archive.
    #[inline]
    pub fn add(&mut self, t: impl Into<ResourceType>, id: u32, data: impl Read) -> io::Result<()> {
        self.add_inner(t.into(), id, data)
    }

    fn add_inner(&mut self, t: ResourceType, id: u32, mut data: impl Read) -> io::Result<()> {
        let res = DRSResource {
            id,
            offset: 0, // TBD
            size: 0,   // TBD
        };

        let res = self
            .strategy
            .add_resource(&mut self.inner, t, res, &mut data)?;

        match self
            .inner
            .tables
            .iter_mut()
            .find(|table| table.resource_type == t)
        {
            Some(table) => {
                table.add(res);
            }
            None => {
                let mut table = DRSTable::new(t, 0, 0);
                table.add(res);
                self.inner.tables.push(table);
            }
        }

        Ok(())
    }

    /// Finish any writes that still need to happen and return the file handle.
    pub fn flush(mut self) -> io::Result<W> {
        assert!(
            self.inner.tables.len() < u32::max_value() as usize,
            "too many tables"
        );
        self.inner.header.num_resource_types = self.inner.tables.len() as u32;

        self.strategy.close(&mut self.inner)?;

        Ok(self.inner.into_inner())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    /// A drs archive with a single text file containing the ASCII text "example test file".
    ///
    /// Copyright
    /// Version 1.00
    /// Password tribe
    /// Resource types 1
    /// First file offset 88
    /// Table "txt", offset 76, resources 1
    /// File 1, offset 88, size 17
    static ONE_FILE: &[u8] = b"Copyright (c) 1997 Ensemble Studios.\x1a\x00\x00\x001.00tribe\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00\x58\x00\x00\x00 txt\x4C\x00\x00\x00\x01\x00\x00\x00\x01\x00\x00\x00\x58\x00\x00\x00\x11\x00\x00\x00example test file";

    #[test]
    fn one_file_reserve() -> Result<(), Box<dyn std::error::Error>> {
        let output = Cursor::new(vec![]);
        let mut drs = DRSWriter::new(output, ReserveDirectoryStrategy::new(1, 1))?;
        drs.add("txt", 1, "example test file".as_bytes())?;
        let output = drs.flush()?.into_inner();
        assert_eq!(output, ONE_FILE.to_vec());
        Ok(())
    }

    #[test]
    fn one_file_memory() -> Result<(), Box<dyn std::error::Error>> {
        let output = Cursor::new(vec![]);
        let mut drs = DRSWriter::new(output, InMemoryStrategy::default())?;
        drs.add("txt", 1, "example test file".as_bytes())?;
        let output = drs.flush()?.into_inner();
        assert_eq!(output, ONE_FILE.to_vec());
        Ok(())
    }
}
