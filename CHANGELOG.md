# genie-rs change log

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](http://semver.org/).

## 0.5.0
* **(breaking)** scx: fix Age of Empires 2: Definitive Edition tile data types. `MapTile.layered_terrain` now contains a u16 instead of a u8.
* **(breaking)** scx: read versioned map data from Age of Empires 2: Definitive Edition.
* **(breaking)** cpx: update genie-scx to v4.0.0.
* cpx: support reading and writing Age of Empires 2: Definitive Edition campaign files. (#22)
* rec: fix small action buffer optimisation.

## 0.4.0
* **(breaking)** scx: support Age of Empires 2: Definitive Edition scenario files. (#28)
* **(breaking)** scx: change `DataStruct::from(&mut Read)` methods to `DataStruct::read_from(impl Read)`. (#28)
* **(breaking)** cpx: update genie-scx to v3.0.0.
* cpx: support reading and writing AoE1: Definitive Edition campaign files. (#18)
* dat: Add a `.dat` file reader with support for The Conquerors and the HD Edition. It has some writing support but makes no guarantees yet.
* drs: make `ResourceType` act more like a `&str`. (#15)
* lang: disable unused `pelite` features for leaner DLL reading.
* rec: Add a recorded game file reader with support for Age of Kings and The Conquerors. (#8)
* scx: support writing embedded AI information and triggers. (#17, #28)
* Use `thiserror` for custom error types. (#27)

## 0.3.0
* **(breaking)** genie: Raise minimum language version requirement to Rust 1.34, for the `TryFrom` trait.
* **(breaking)** scx: Add descriptive error types.
* **(breaking)** cpx: Add descriptive error types.
* **(breaking)** drs: Add descriptive error types.
* **(breaking)** hki: Add non-destructive update functions for binding hotkeys. (@twestura in #3)
* **(breaking)** lang: Overhaul APIs. (@twestura in #3)
* **(breaking)** pal: Replace `chariot_palette` with custom jascpal crate, adding support for writing palette files.
* drs: Add a DRS file writer.
* cpx: Detect and convert non-UTF8 encodings.
* drs: find resources faster using binary search. (#6)

## 0.2.0
* Add a cpx file writer.
* Import genie-drs, for reading .DRS files.
* Add read/write support for .ini and HD Edition key-value language files, and read support for .dll language files.

## 0.1.0
* Initial release.
