# genie-drs

[![docs.rs](https://img.shields.io/badge/docs.rs-genie--drs-blue?style=flat-square&color=blue)](https://docs.rs/genie-drs/)
[![crates.io](https://img.shields.io/crates/v/genie-drs.svg?style=flat-square&color=orange)](https://crates.io/crates/genie-drs)
[![GitHub license](https://img.shields.io/github/license/SiegeEngineers/genie-rs?style=flat-square&color=darkred)](https://github.com/SiegeEngineers/genie-rs/blob/default/LICENSE.md)
![MSRV](https://img.shields.io/badge/MSRV-1.64.0%2B-blue?style=flat-square)

Read .drs archive files from the Genie Engine, used in Age of Empires 1/2 and SWGB

## About DRS

.drs is the resource archive file format for the Genie Engine, used by Age of Empires 1/2 and
Star Wars: Galactic Battlegrounds. .drs files contain tables, each of which contain resources
of a single type. Resources are identified by a numeric identifier.

This crate only supports reading files right now.

## Install

Add to Cargo.toml:

```toml
[dependencies]
genie-drs = "^0.2.1"
```

## Example

```rust
use std::fs::File;
use genie_drs::DRSReader;

let mut file = File::open("test.drs")?;
let drs = DRSReader::new(&mut file)?;

for table in drs.tables() {
    for resource in table.resources() {
        let content = drs.read_resource(&mut file, table.resource_type, resource.id)?;
        println!("{}: {:?}", resource.id, std::str::from_utf8(&content)?);
    }
}
```

## Wishlist

- An API that doesn't require passing in the file handle manually
- A [file mapping](https://en.wikipedia.org/wiki/Memory-mapped_file) counterpart for the `read_resource` API, using [memmap](https://crates.io/crates/memmap) probably.

## License

[GPL-3.0 or later](./LICENSE.md)
