use ffi_support::FfiStr;
use genie_lang::{LangFile, LangFileType, StringKey};
use std::{fs::File, ptr};

fn try_read_file(t: LangFileType, path: FfiStr) -> *mut LangFile {
    if let Some(path) = path.as_opt_str() {
        if let Ok(file) = File::open(path) {
            t.read_from(file)
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

fn try_create_file(path: FfiStr) -> Option<File> {
    path.as_opt_str().and_then(|path| File::create(path).ok())
}

/// Load a .ini language file.
#[no_mangle]
pub extern "C" fn cglang_load_ini(path: FfiStr) -> *mut LangFile {
    try_read_file(LangFileType::Ini, path)
}

/// Load an HD Edition key-value.txt language file.
#[no_mangle]
pub extern "C" fn cglang_load_keyval(path: FfiStr) -> *mut LangFile {
    try_read_file(LangFileType::KeyValue, path)
}

/// Load a classic DLL language file.
#[no_mangle]
pub extern "C" fn cglang_load_dll(path: FfiStr) -> *mut LangFile {
    try_read_file(LangFileType::Dll, path)
}

/// Get an integer-indexed string.
#[no_mangle]
pub extern "C" fn cglang_get(file: *const LangFile, index: u32) -> *const u8 {
    if file.is_null() {
        ptr::null()
    } else {
        unsafe { &*file }
            .get(&StringKey::from(index))
            .map(|s| s.as_ptr())
            .unwrap_or(ptr::null())
    }
}

/// Get a name-indexed string.
#[no_mangle]
pub extern "C" fn cglang_get_named(file: *const LangFile, index: FfiStr) -> *const u8 {
    let index = index.as_opt_str();
    if file.is_null() || index.is_none() {
        ptr::null()
    } else {
        unsafe { &*file }
            .get(&StringKey::from(index.unwrap()))
            .map(|s| s.as_ptr())
            .unwrap_or(ptr::null())
    }
}

/// Save a .ini language file.
#[no_mangle]
pub extern "C" fn cglang_save_ini(file: *mut LangFile, path: FfiStr) -> u32 {
    let mut output = match try_create_file(path) {
        Some(output) => output,
        _ => return 1,
    };
    if let Ok(_) = unsafe { &*file }.write_to_ini(&mut output) {
        return 0;
    }
    return 2;
}

/// Save an HD Edition key-value.txt language file.
#[no_mangle]
pub extern "C" fn cglang_save_keyval(file: *mut LangFile, path: FfiStr) -> u32 {
    let mut output = match try_create_file(path) {
        Some(output) => output,
        _ => return 1,
    };
    if let Ok(_) = unsafe { &*file }.write_to_keyval(&mut output) {
        return 0;
    }
    return 2;
}

/// Free all language file resources.
#[no_mangle]
pub extern "C" fn cglang_free(file: *mut LangFile) {
    if !file.is_null() {
        let file = unsafe { Box::from_raw(file) };
        drop(file);
    }
}
