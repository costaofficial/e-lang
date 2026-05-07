use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

fn c_str(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

fn load(s: *const c_char) -> String {
    if s.is_null() { String::new() } else { unsafe { CStr::from_ptr(s) }.to_string_lossy().to_string() }
}

#[no_mangle]
pub extern "C" fn e_exists(path: *const c_char) -> *mut c_char {
    c_str(if Path::new(&load(path)).exists() { "true" } else { "false" })
}

#[no_mangle]
pub extern "C" fn e_size(path: *const c_char) -> *mut c_char {
    let size = std::fs::metadata(&load(path)).map(|m| m.len()).unwrap_or(0);
    c_str(&format!("{}", size))
}

#[no_mangle]
pub extern "C" fn e_copy(from: *const c_char, to: *const c_char) -> *mut c_char {
    c_str(&match std::fs::copy(&load(from), &load(to)) {
        Ok(n) => format!("{{\"ok\": true, \"bytes\": {}}}", n),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    })
}

#[no_mangle]
pub extern "C" fn e_delete(path: *const c_char) -> *mut c_char {
    c_str(&match std::fs::remove_file(&load(path)) {
        Ok(_) => "true".to_string(),
        Err(e) => format!("false: {}", e),
    })
}

#[no_mangle]
pub extern "C" fn e_free(s: *mut c_char) {
    if !s.is_null() { unsafe { let _ = CString::from_raw(s); } }
}
