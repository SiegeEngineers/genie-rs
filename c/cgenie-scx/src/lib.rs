//! C API for the genie-scx crate.
//!
//! The C API does not offer detailed reading and writing capabilities, only premade conversions.
//!
//! ## Example
//! ```c
//! #include <cgenie/scx.h>
//! int main() {
//!     cgenie_scx_t scenario = cgscx_load("file.aoe2scenario");
//!     if (scenario == NULL) return 1;
//!     cgenie_scx_result_t result = cgscx_convert_hd_to_wk(scenario);
//!     if (result == cgscxOk) result = cgscx_save(scenario, "file.scx");
//!     switch (result) {
//!         case cgscxErrCreateFile:
//!         case cgscxErrConvert:
//!         case cgscxErrSerialize:
//!             fprintf(stderr, "Could not convert scenario");
//!             return 1;
//!         case cgscxOk:
//!             return 0;
//!     }
//! }
//! ```

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(unused)]

use ffi_support::FfiStr;
use genie_scx::convert::{AoCToWK, AutoToWK, HDToWK};
use genie_scx::{Scenario, VersionBundle};
use std::fs::File;
use std::io::Cursor;
use std::ptr;

/// Open and read a scenario file.
///
/// # Examples
/// ```c
/// #include <cgenie/scx.h>
///
/// int main () {
///   cgenie_scx_t scenario = cgscx_load("filename.scx");
///   if (scenario == NULL) {
///     return 1;
///   }
///   cgscx_free(scenario);
///   return 0;
/// }
/// ```
#[no_mangle]
pub extern "C" fn cgscx_load(path: FfiStr<'_>) -> *mut Scenario {
    if let Some(path) = path.as_opt_str() {
        if let Ok(mut file) = File::open(path) {
            Scenario::from(&mut file)
                .map(Box::new)
                .map(Box::into_raw)
                .unwrap_or(ptr::null_mut())
        } else {
            ptr::null_mut()
        }
    } else {
        ptr::null_mut()
    }
}

/// Read a scenario file from a byte array.
///
/// # Examples
/// ```c
/// #include <cgenie/scx.h>
///
/// int main () {
///   FILE* file = fopen("filename.scx", "rb");
///   fseek(file, 0L, SEEK_END);
///   long size = ftell(file);
///   fseek(file, 0L, SEEK_SET);
///   unsigned char* bytes = calloc(1, size);
///   fread(bytes, 1, size, file);
///
///   cgenie_scx_t scenario = cgscx_load_mem(bytes);
///   if (scenario == NULL) {
///     return 1;
///   }
///   cgscx_free(scenario);
///   return 0;
/// }
/// ```
#[no_mangle]
pub extern "C" fn cgscx_load_mem(input: *const u8, size: usize) -> *mut Scenario {
    let slice = unsafe { std::slice::from_raw_parts(input, size) };
    let mut cursor = Cursor::new(slice);
    Scenario::from(&mut cursor)
        .map(Box::new)
        .map(Box::into_raw)
        .unwrap_or(ptr::null_mut())
}

/// Convert an AoC scenario file to WololoKingdoms.
#[no_mangle]
pub extern "C" fn cgscx_convert_aoc_to_wk(scenario: *mut Scenario) -> u32 {
    if scenario.is_null() {
        return 1;
    }
    let scenario = unsafe { &mut *scenario };

    let converter = AoCToWK::default();
    if let Err(_) = converter.convert(scenario) {
        return 3;
    }

    return 0;
}

/// Convert an HD Edition scenario file to WololoKingdoms.
#[no_mangle]
pub extern "C" fn cgscx_convert_hd_to_wk(scenario: *mut Scenario) -> u32 {
    if scenario.is_null() {
        return 1;
    }
    let scenario = unsafe { &mut *scenario };

    let converter = HDToWK::default();
    if let Err(_) = converter.convert(scenario) {
        return 3;
    }

    return 0;
}

/// Try to convert any scenario file to WololoKingdoms.
#[no_mangle]
pub extern "C" fn cgscx_convert_to_wk(scenario: *mut Scenario) -> u32 {
    if scenario.is_null() {
        return 1;
    }
    let scenario = unsafe { &mut *scenario };

    let converter = AutoToWK::default();
    if let Err(_) = converter.convert(scenario) {
        return 3;
    }

    return 0;
}

/// Save the scenario to a file.
#[no_mangle]
pub extern "C" fn cgscx_save(
    scenario: *mut Scenario,
    version: FfiStr<'_>,
    path: FfiStr<'_>,
) -> u32 {
    if scenario.is_null() {
        return 1;
    }
    let scenario = unsafe { &*scenario };

    let version = match version.as_opt_str() {
        Some("aoe") => VersionBundle::aoe(),
        Some("ror") => VersionBundle::ror(),
        Some("aoc") => VersionBundle::aoc(),
        Some("hd") => VersionBundle::hd_edition(),
        Some("wk") => VersionBundle::userpatch_15(),
        Some(_) => return 5,
        None => scenario.version().clone(),
    };
    if let Some(path) = path.as_opt_str() {
        if let Ok(mut file) = File::create(path) {
            if let Ok(_) = scenario.write_to_version(&mut file, &version) {
                0
            } else {
                4
            }
        } else {
            2
        }
    } else {
        1
    }
}
