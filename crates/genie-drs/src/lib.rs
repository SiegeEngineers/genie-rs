//! .drs is the resource archive file format for the Genie Engine, used by Age of Empires 1/2 and
//! Star Wars: Galactic Battlegrounds. .drs files contain tables, each of which contain resources
//! of a single type. Resources are identified by a numeric identifier.
//!
//! ## Example
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use std::fs::File;
//! use genie_drs::DRSReader;
//!
//! let mut file = File::open("test.drs")?;
//! let drs = DRSReader::new(&mut file)?;
//!
//! for table in drs.tables() {
//!     for resource in table.resources() {
//!         let content = drs.read_resource(&mut file, table.resource_type, resource.id)?;
//!         println!("{}: {:?}", resource.id, std::str::from_utf8(&content)?);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused)]

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use sorted_vec::SortedVec;
use std::io::{Error, Read, Write};
use std::slice;
use std::str;

mod read;
mod write;

pub use read::DRSReader;
pub use write::{DRSWriter, InMemoryStrategy, ReserveDirectoryStrategy, Strategy as WriteStrategy};

/// A DRS version string.
type DRSVersion = [u8; 4];

/// A resource type name.
///
/// In a .drs archive, type names are represented as 4 bytes. They are laid out in reverse order and
/// padded with ASCII space characters (`' '`). For example, the "slp" resource type is stored as `" pls"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceType([u8; 4]);
impl ResourceType {
    #[inline]
    fn write_to<W: Write>(self, output: &mut W) -> Result<(), Error> {
        output.write_all(&self.0)?;
        Ok(())
    }
}

impl ToString for ResourceType {
    fn to_string(&self) -> String {
        let mut bytes = [0 as u8; 4];
        bytes.clone_from_slice(&self.0);
        bytes.reverse();
        str::from_utf8(&bytes).unwrap().trim().to_string()
    }
}

/// An error occurred while parsing a resource type.
///
/// This may be caused by:
///   - The input string not being 4 characters long
#[derive(Debug)]
pub struct ParseResourceTypeError;

/// Parse a resource type from a string, with error handling.
impl core::str::FromStr for ResourceType {
    type Err = ParseResourceTypeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() > 4 {
            Err(ParseResourceTypeError)
        } else {
            Ok(bytes.into())
        }
    }
}

impl From<[u8; 4]> for ResourceType {
    fn from(u: [u8; 4]) -> Self {
        Self(u)
    }
}

/// Parse a resource type from a byte slice, panics if the slice is too long to fit.
impl From<&[u8]> for ResourceType {
    fn from(u: &[u8]) -> Self {
        assert!(u.len() <= 4);
        let mut bytes = [b' '; 4];
        (&mut bytes[0..u.len()]).copy_from_slice(u);
        bytes.reverse();
        Self(bytes)
    }
}

/// Parse a resource type from a string, panics if the string is too long to fit (>4 bytes).
impl From<&str> for ResourceType {
    fn from(s: &str) -> Self {
        s.as_bytes().into()
    }
}

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

impl Default for DRSHeader {
    fn default() -> Self {
        Self {
            banner_msg: *b"Copyright (c) 1997 Ensemble Studios.\x1a\x00\x00\x00",
            version: *b"1.00",
            password: *b"tribe\x00\x00\x00\x00\x00\x00\x00",
            num_resource_types: 0,
            directory_size: 0,
        }
    }
}

impl DRSHeader {
    #[inline]
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

    #[inline]
    fn write_to<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        output.write_all(&self.banner_msg)?;
        output.write_all(&self.version)?;
        output.write_all(&self.password)?;
        output.write_u32::<LE>(self.num_resource_types)?;
        output.write_u32::<LE>(self.directory_size)?;
        Ok(())
    }
}

impl std::fmt::Debug for DRSHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    /// Resource IDs.
    resource_ids: SortedVec<u32>,
}

impl DRSTable {
    fn new(resource_type: ResourceType, offset: u32, num_resources: u32) -> Self {
        Self {
            resource_type,
            offset,
            num_resources,
            resources: Default::default(),
            resource_ids: Default::default(),
        }
    }

    /// Read a DRS table header from a `Read`able handle.
    #[inline]
    fn from<R: Read>(source: &mut R) -> Result<DRSTable, Error> {
        let mut resource_type = [0 as u8; 4];
        source.read_exact(&mut resource_type)?;
        let offset = source.read_u32::<LE>()?;
        let num_resources = source.read_u32::<LE>()?;
        Ok(DRSTable::new(resource_type.into(), offset, num_resources))
    }

    #[inline]
    fn write_to<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        self.resource_type.write_to(output)?;
        output.write_u32::<LE>(self.offset)?;
        output.write_u32::<LE>(self.num_resources)?;
        Ok(())
    }

    /// Read the table itself.
    #[inline]
    fn read_resources<R: Read>(&mut self, source: &mut R) -> Result<(), Error> {
        for _ in 0..self.num_resources {
            let resource = DRSResource::from(source)?;
            let _discard = self.resource_ids.insert(resource.id);
            self.resources.push(resource);
        }
        Ok(())
    }

    /// Get the number of resources in this table.
    #[inline]
    pub fn len(&self) -> usize {
        self.num_resources as usize
    }

    /// Check if the table contains no resources.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.num_resources == 0
    }

    /// Iterate over the resources in this table.
    #[inline]
    pub fn resources(&self) -> DRSResourceIterator<'_> {
        self.resources.iter()
    }

    /// Find a resource by ID.
    #[inline]
    pub fn get_resource(&self, id: u32) -> Option<&DRSResource> {
        self.resource_ids
            .binary_search(&id)
            .ok()
            .map(|index| &self.resources[index])
    }

    #[inline]
    pub fn resource_ext(&self) -> String {
        self.resource_type.to_string()
    }

    #[inline]
    pub(crate) fn add(&mut self, res: DRSResource) -> &mut DRSResource {
        self.resources.push(res);
        self.num_resources += 1;
        self.resources.last_mut().unwrap()
    }
}

impl std::fmt::Debug for DRSTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DRSTable {{ resource_type: '{}', offset: {}, num_resources: {} }}",
            self.resource_type.to_string(),
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
    #[inline]
    fn from<R: Read>(source: &mut R) -> Result<DRSResource, Error> {
        let id = source.read_u32::<LE>()?;
        let offset = source.read_u32::<LE>()?;
        let size = source.read_u32::<LE>()?;
        Ok(DRSResource { id, offset, size })
    }

    #[inline]
    fn write_to<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        output.write_u32::<LE>(self.id)?;
        output.write_u32::<LE>(self.offset)?;
        output.write_u32::<LE>(self.size)?;
        Ok(())
    }
}

/// An iterator over DRS table metadata structs.
pub type DRSTableIterator<'a> = slice::Iter<'a, DRSTable>;
/// An iterator over DRS resource metadata structs.
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
            ("js".parse().unwrap(), 1, 632),
            ("js".parse().unwrap(), 2, 452),
            ("js".parse().unwrap(), 3, 38),
            ("json".parse().unwrap(), 4, 710),
        ];

        for table in drs.tables() {
            for resource in table.resources() {
                let content = drs
                    .read_resource(&mut file, table.resource_type, resource.id)
                    .unwrap();
                assert_eq!(
                    expected.remove(0),
                    (table.resource_type, resource.id, content.len())
                );
            }
        }
    }
}
