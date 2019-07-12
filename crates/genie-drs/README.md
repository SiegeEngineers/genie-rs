# genie-drs

.drs is the resource archive file format for the Genie Engine, used by Age of Empires 1/2 and
Star Wars: Galactic Battlegrounds. .drs files contain tables, each of which contain resources
of a single type. Resources are identified by a numeric identifier.

This crate only supports reading files right now.

## Install

Add to Cargo.toml:

```toml
[dependencies]
genie-drs = "^0.1.1"
```

## Example

```rust
use std::fs::File;
use genie_drs::DRSReader;

let mut file = File::open("test.drs").unwrap();
let drs = DRSReader::new(&mut file).unwrap();

for table in drs.tables() {
    for resource in table.resources() {
        let content = drs.read_resource(&mut file, table.resource_type, resource.id).unwrap();
        println!("{}: {:?}", resource.id, std::str::from_utf8(&content).unwrap());
    }
}
```

## Wishlist

- An API that doesn't require passing in the file handle manually
- A [file mapping](https://en.wikipedia.org/wiki/Memory-mapped_file) counterpart for the `read_resource` API, using [memmap](https://crates.io/crates/memmap) probably.

## License

[GPL-3.0 or later](./LICENSE.md)
