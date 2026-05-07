use std::ffi::{CStr, CString};
use std::os::raw::c_char;

fn c_str(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

fn load(s: *const c_char) -> String {
    if s.is_null() { String::new() } else { unsafe { CStr::from_ptr(s) }.to_string_lossy().to_string() }
}

#[no_mangle]
pub extern "C" fn e_open(path: *const c_char) -> *mut c_char {
    let p = load(path);
    c_str(&format!("{{\"ok\": true, \"path\": \"{}\"}}", p))
}

#[no_mangle]
pub extern "C" fn e_query(handle: *const c_char, sql: *const c_char) -> *mut c_char {
    let h = load(handle);
    let s = load(sql);
    c_str(&format!("{{\"query\": \"{}\", \"handle\": \"{}\"}}", s.replace('"', "\\\""), h))
}

#[no_mangle]
pub extern "C" fn e_free(s: *mut c_char) {
    if !s.is_null() { unsafe { let _ = CString::from_raw(s); } }
}
