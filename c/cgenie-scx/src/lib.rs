use ffi_support::FfiStr;
use genie_scx::{convert::HDToWK, Scenario};
use std::{fs::File, io::Cursor, ptr};

/// Open and read a scenario file.
#[no_mangle]
pub extern "C" fn cgscx_load(path: FfiStr) -> *mut Scenario {
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
#[no_mangle]
pub extern "C" fn cgscx_load_mem(input: *const u8, size: usize) -> *mut Scenario {
    let slice = unsafe { std::slice::from_raw_parts(input, size) };
    let mut cursor = Cursor::new(slice);
    Scenario::from(&mut cursor)
        .map(Box::new)
        .map(Box::into_raw)
        .unwrap_or(ptr::null_mut())
}

/// Convert an HD Edition scenario file to WololoKingdoms.
pub extern "C" fn cgscx_convert_hd_to_wk(scenario: *mut Scenario) -> u32 {
    if scenario.is_null() {
        return 1;
    }

    let converter = HDToWK::default();
    if let Err(_) = converter.convert(unsafe { &mut *scenario }) {
        return 3;
    }

    return 0;
}

/// Save the scenario to a file.
#[no_mangle]
pub extern "C" fn cgscx_save(scenario: *mut Scenario, path: FfiStr) -> u32 {
    if let Some(path) = path.as_opt_str() {
        if let Ok(mut file) = File::create(path) {
            if let Ok(_) = unsafe { &*scenario }.write_to(&mut file) {
                0
            } else {
                3
            }
        } else {
            2
        }
    } else {
        1
    }
}
