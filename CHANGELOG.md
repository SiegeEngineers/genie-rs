# genie-rs change log

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](http://semver.org/).

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
