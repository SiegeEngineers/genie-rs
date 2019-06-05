//! .drs is the resource archive file format for the Genie Engine, used by Age of Empires 1/2 and
//! Star Wars: Galactic Battlegrounds. .drs files contain tables, each of which contain resources
//! of a single type. Resources are identified by a numeric identifier.
//!
//! ## Example
//!
//! ```rust
//! use std::fs::File;
//! use genie_drs::DRSReader;
//!
//! let mut file = File::open("test.drs").unwrap();
//! let drs = DRSReader::new(&mut file).unwrap();
//!
//! for table in drs.tables() {
//!     for resource in table.resources() {
//!         let content = drs.read_resource(&mut file, table.resource_type, resource.id).unwrap();
//!         println!("{}: {:?}", resource.id, std::str::from_utf8(&content).unwrap());
//!     }
//! }
//! ```

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Error, Read, Write};
use std::slice;
use std::str;

mod read;
mod write;

pub use read::DRSReader;
pub use write::{DRSWriter, Strategy as WriteStrategy};

/// A DRS version string.
type DRSVersion = [u8; 4];

/// A resource type name.
pub type ResourceType = [u8; 4];

/// The DRS archive header.
pub struct DRSHeader {
    /// A copyright message.
    banner_msg: [u8; 40],
    /// File version. (always "1.00")
    version: DRSVersion,
    /// File password / identifier.
    password: [u8; 12],
    /// The amount of resource types (tables).
    num_resource_types: u32,
    /// Size in bytes of the metadata and tables. Resource contents start at this offset.
    directory_size: u32,
}

impl DRSHeader {
    /// Read a DRS archive header from a `Read`able handle.
    fn from<R: Read>(source: &mut R) -> Result<DRSHeader, Error> {
        let mut banner_msg = [0 as u8; 40];
        let mut version = [0 as u8; 4];
        let mut password = [0 as u8; 12];
        source.read_exact(&mut banner_msg)?;
        source.read_exact(&mut version)?;
        source.read_exact(&mut password)?;
        let num_resource_types = source.read_u32::<LE>()?;
        let directory_size = source.read_u32::<LE>()?;
        Ok(DRSHeader {
            banner_msg,
            version,
            password,
            num_resource_types,
            directory_size,
        })
    }

    pub fn write_to<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        output.write_all(&self.banner_msg)?;
        output.write_all(&self.version)?;
        output.write_all(&self.password)?;
        output.write_u32::<LE>(self.num_resource_types)?;
        output.write_u32::<LE>(self.directory_size)?;
        Ok(())
    }
}

impl std::fmt::Debug for DRSHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
           "DRSHeader {{ banner_msg: '{}', version: '{}', password: '{}', num_resource_types: {}, directory_size: {} }}",
           str::from_utf8(&self.banner_msg).unwrap(),
           str::from_utf8(&self.version).unwrap(),
           str::from_utf8(&self.password).unwrap(),
           self.num_resource_types,
           self.directory_size
        )
    }
}

/// A table containing resource entries.
pub struct DRSTable {
    /// Type of the resource as a little-endian char array.
    pub resource_type: ResourceType,
    /// Offset in the DRS archive where this table's resource entries can be found.
    offset: u32,
    /// Number of resource entries in this table.
    num_resources: u32,
    /// Resources.
    resources: Vec<DRSResource>,
}

impl DRSTable {
    /// Read a DRS table header from a `Read`able handle.
    fn from<R: Read>(source: &mut R) -> Result<DRSTable, Error> {
        let mut resource_type = [0 as u8; 4];
        source.read_exact(&mut resource_type)?;
        let offset = source.read_u32::<LE>()?;
        let num_resources = source.read_u32::<LE>()?;
        Ok(DRSTable {
            resource_type,
            offset,
            num_resources,
            resources: vec![],
        })
    }

    fn write_to<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        output.write_all(&self.resource_type)?;
        output.write_u32::<LE>(self.offset)?;
        output.write_u32::<LE>(self.num_resources)?;
        Ok(())
    }

    /// Read the table itself.
    fn read_resources<R: Read>(&mut self, source: &mut R) -> Result<(), Error> {
        for _ in 0..self.num_resources {
            self.resources.push(DRSResource::from(source)?);
        }
        Ok(())
    }

    /// Get the number of resources in this table.
    pub fn len(&self) -> usize {
        self.num_resources as usize
    }

    /// Check if the table contains no resources.
    pub fn is_empty(&self) -> bool {
        self.num_resources == 0
    }

    /// Iterate over the resources in this table.
    pub fn resources(&self) -> DRSResourceIterator {
        self.resources.iter()
    }

    /// Find a resource by ID.
    pub fn get_resource(&self, id: u32) -> Option<&DRSResource> {
        self.resources().find(|resource| resource.id == id)
    }

    pub fn resource_ext(&self) -> String {
        let mut resource_type = [0 as u8; 4];
        resource_type.clone_from_slice(&self.resource_type);
        resource_type.reverse();
        str::from_utf8(&resource_type).unwrap().trim().to_string()
    }

    pub(crate) fn add(&mut self, res: DRSResource) -> &mut DRSResource {
        self.resources.push(res);
        self.num_resources += 1;
        self.resources.last_mut().unwrap()
    }
}

impl std::fmt::Debug for DRSTable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut resource_type = [0 as u8; 4];
        resource_type.clone_from_slice(&self.resource_type);
        resource_type.reverse();
        write!(
            f,
            "DRSTable {{ resource_type: '{}', offset: {}, num_resources: {} }}",
            str::from_utf8(&resource_type).unwrap(),
            self.offset,
            self.num_resources
        )
    }
}

/// A single resource in a DRS archive.
#[derive(Debug)]
pub struct DRSResource {
    /// The resource ID.
    pub id: u32,
    /// The offset into the DRS archive where the resource can be found.
    offset: u32,
    /// The size in bytes of the resource.
    pub size: u32,
}

impl DRSResource {
    /// Read DRS resource metadata from a `Read`able handle.
    fn from<R: Read>(source: &mut R) -> Result<DRSResource, Error> {
        let id = source.read_u32::<LE>()?;
        let offset = source.read_u32::<LE>()?;
        let size = source.read_u32::<LE>()?;
        Ok(DRSResource { id, offset, size })
    }

    fn write_to<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        output.write_u32::<LE>(self.id)?;
        output.write_u32::<LE>(self.offset)?;
        output.write_u32::<LE>(self.size)?;
        Ok(())
    }
}

pub type DRSTableIterator<'a> = slice::Iter<'a, DRSTable>;
pub type DRSResourceIterator<'a> = slice::Iter<'a, DRSResource>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn it_works() {
        let mut file = File::open("test.drs").unwrap();
        let drs = DRSReader::new(&mut file).unwrap();
        let mut expected = vec![
            // (reversed_type, id, size)
            (b"  sj", 1, 632),
            (b"  sj", 2, 452),
            (b"  sj", 3, 38),
            (b"nosj", 4, 710),
        ];

        for table in drs.tables() {
            for resource in table.resources() {
                let content = drs
                    .read_resource(&mut file, table.resource_type, resource.id)
                    .unwrap();
                assert_eq!(
                    expected.remove(0),
                    (&table.resource_type, resource.id, content.len())
                );
            }
        }
    }
}
