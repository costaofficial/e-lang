use std::ffi::{CStr, CString};
use std::os::raw::c_char;

fn c_str(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

fn load(s: *const c_char) -> String {
    if s.is_null() { String::new() } else { unsafe { CStr::from_ptr(s) }.to_string_lossy().to_string() }
}

#[no_mangle]
pub extern "C" fn e_parse(input: *const c_char) -> *mut c_char {
    let s = load(input);
    let result = match serde_json::from_str::<serde_json::Value>(&s) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or(s),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    };
    c_str(&result)
}

#[no_mangle]
pub extern "C" fn e_stringify(input: *const c_char) -> *mut c_char {
    let s = load(input);
    let result = match serde_json::from_str::<serde_json::Value>(&s) {
        Ok(v) => v.to_string(),
        Err(_) => format!("\"{}\"", s.replace('"', "\\\"")),
    };
    c_str(&result)
}

#[no_mangle]
pub extern "C" fn e_free(s: *mut c_char) {
    if !s.is_null() { unsafe { let _ = CString::from_raw(s); } }
}
