use std::ffi::{CStr, CString};
use std::os::raw::c_char;

fn c_str(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

fn load(s: *const c_char) -> String {
    if s.is_null() { String::new() } else { unsafe { CStr::from_ptr(s) }.to_string_lossy().to_string() }
}

#[no_mangle]
pub extern "C" fn e_get(url: *const c_char) -> *mut c_char {
    let url = load(url);
    match ureq::get(&url).call() {
        Ok(resp) => match resp.into_string() {
            Ok(body) => c_str(&format!("{{\"ok\": true, \"body\": {}}}", serde_json::to_string(&body).unwrap_or_else(|_| format!("\"{}\"", body)))),
            Err(e) => c_str(&format!("{{\"error\": \"read failed: {}\"}}", e)),
        },
        Err(e) => c_str(&format!("{{\"error\": \"{}\"}}", e)),
    }
}

#[no_mangle]
pub extern "C" fn e_post(url: *const c_char, body: *const c_char) -> *mut c_char {
    let url = load(url);
    let data = load(body);
    match ureq::post(&url).send_string(&data) {
        Ok(resp) => match resp.into_string() {
            Ok(b) => c_str(&b),
            Err(e) => c_str(&format!("{{\"error\": \"read failed: {}\"}}", e)),
        },
        Err(e) => c_str(&format!("{{\"error\": \"{}\"}}", e)),
    }
}

#[no_mangle]
pub extern "C" fn e_free(s: *mut c_char) {
    if !s.is_null() { unsafe { let _ = CString::from_raw(s); } }
}
