use ffi_support::FfiStr;
use genie_cpx::Campaign;
use std::{fs::File, io::{Read, Seek, Cursor}, ptr};

/// Open and read a campaign file.
#[no_mangle]
pub extern "C" fn cgcpx_load(path: FfiStr) -> *mut Campaign {
    if let Some(path) = path.as_opt_str() {
        if let Ok(mut file) = File::open(path) {
            Campaign::from(&mut file)
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

/// Read a campaign file from a byte array.
#[no_mangle]
pub extern "C" fn cgcpx_load_mem(input: *const u8, size: usize) -> *mut Campaign {
    let slice = unsafe { std::slice::from_raw_parts(input, size) };
    let mut cursor = Cursor::new(slice);
    Campaign::from(&mut cursor)
        .map(Box::new)
        .map(Box::into_raw)
        .unwrap_or(ptr::null_mut())
}
