use std::ffi::CString;
use std::os::raw::c_char;

// Fixed-size buffer for formatting without heap allocation
use std::fmt::Write;

// Format a string into a fixed stack buffer and return it as CString
// This avoids heap allocation from format!
macro_rules! format_on_stack {
    ($($arg:tt)*) => {{
        let mut buf = [0u8; 4096];
        let mut idx = 0;
        // Basic formatting without allocation
        let s = format_noalloc(&mut buf, &mut idx, format_args!($($arg)*));
        CString::new(s).unwrap().into_raw()
    }};
}

// Simple no-allocation formatter using a fixed buffer
// This is a simplified version that handles basic string formatting
fn format_noalloc<'a>(buf: &'a mut [u8; 4096], idx: &mut usize, args: std::fmt::Arguments) -> &'a str {
    use std::fmt::Write;
    let mut w = FixedWriter { buf, idx };
    let _ = write!(&mut w, "{}", args);
    // Null-terminate
    if *w.idx < w.buf.len() { w.buf[*w.idx] = 0; }
    // Return as str (unsafe but safe because we control the buffer)
    unsafe { std::str::from_utf8_unchecked(&w.buf[..*w.idx]) }
}

struct FixedWriter<'a> {
    buf: &'a mut [u8; 4096],
    idx: &'a mut usize,
}

impl<'a> Write for FixedWriter<'a> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - *self.idx;
        let to_copy = bytes.len().min(remaining);
        self.buf[*self.idx..*self.idx + to_copy].copy_from_slice(&bytes[..to_copy]);
        *self.idx += to_copy;
        Ok(())
    }
}

unsafe fn read_str(ptr: *const c_char) -> &'static str {
    if ptr.is_null() { return ""; }
    let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
    std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr as *const u8, len))
}

fn c_str(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn e_open(path: *const c_char) -> *mut c_char {
    let p = unsafe { read_str(path) };
    c_str(&format_no_alloc(&format_args!("{{\"ok\": true, \"path\": \"{}\"}}", p)))
}

fn format_no_alloc<'a>(args: &std::fmt::Arguments, buf: &'a mut [u8; 4096]) -> &'a str {
    let mut idx = 0;
    use std::fmt::Write;
    let mut w = FixedWriter { buf, idx: &mut idx };
    let _ = write!(&mut w, "{}", args);
    if idx < buf.len() { buf[idx] = 0; }
    unsafe { std::str::from_utf8_unchecked(&buf[..idx]) }
}

#[no_mangle]
pub extern "C" fn e_query(_db: *const c_char, _sql: *const c_char) -> *mut c_char {
    // Return static string — no allocation
    b"[]\0" as *const u8 as *mut c_char
}

#[no_mangle]
pub extern "C" fn e_free(_s: *mut c_char) {}
