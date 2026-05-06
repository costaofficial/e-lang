use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn e_hello(input: *const c_char) -> *mut c_char {
    let msg = if input.is_null() {
        "world".to_string()
    } else {
        unsafe { CStr::from_ptr(input) }.to_string_lossy().to_string()
    };
    let result = format!("Hello, {}! From Rust plugin.", msg);
    CString::new(result).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn e_double(n: i64) -> i64 {
    n * 2
}

#[no_mangle]
pub extern "C" fn e_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}
