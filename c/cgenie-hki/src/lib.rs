use ffi_support::FfiStr;
use genie_hki::HotkeyInfo;
use std::{fs::File, io::Cursor, ptr};

/// Open and read a hotkey file.
#[no_mangle]
pub extern "C" fn cghki_load(path: FfiStr) -> *mut HotkeyInfo {
    if let Some(path) = path.as_opt_str() {
        if let Ok(mut file) = File::open(path) {
            HotkeyInfo::from(&mut file)
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

/// Read a hotkey file from a byte array.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn cghki_load_mem(input: *const u8, size: usize) -> *mut HotkeyInfo {
    let slice = unsafe { std::slice::from_raw_parts(input, size) };
    let mut cursor = Cursor::new(slice);
    HotkeyInfo::from(&mut cursor)
        .map(Box::new)
        .map(Box::into_raw)
        .unwrap_or(ptr::null_mut())
}

/// Save the hotkeys to a file.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn cghki_save(hki: *mut HotkeyInfo, path: FfiStr) -> u32 {
    if let Some(path) = path.as_opt_str() {
        if let Ok(mut file) = File::create(path) {
            if unsafe { &*hki }.write_to(&mut file).is_ok() {
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
