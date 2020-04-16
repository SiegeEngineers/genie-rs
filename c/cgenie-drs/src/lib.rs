use ffi_support::FfiStr;
use genie_drs::{DRSReader, DRSWriter};
use std::{fs::File, ptr};

type DRSR = (File, DRSReader);

/// Open a drs archive.
#[no_mangle]
pub extern "C" fn cgdrs_load(path: FfiStr) -> *mut DRSR {
    if let Some(path) = path.as_opt_str() {
        if let Ok(mut file) = File::open(path) {
            DRSReader::new(&mut file)
                .map(move |drs| (file, drs))
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

/// Close a drs archive.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn cgdrs_free(drs: *mut DRSR) {
    let pair = unsafe { Box::from_raw(drs) };
    drop(pair);
}
