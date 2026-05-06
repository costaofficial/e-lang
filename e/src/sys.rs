use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub struct PluginManager {
    loaded: HashMap<String, Plugin>,
}

struct Plugin {
    _path: String,
    library: libloading::Library,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager { loaded: HashMap::new() }
    }

    pub fn load(&mut self, path: &str) -> Result<(), String> {
        if self.loaded.contains_key(path) {
            return Ok(());
        }
        let lib = unsafe {
            libloading::Library::new(path)
                .map_err(|e| format!("cannot load '{}': {}", path, e))?
        };
        self.loaded.insert(path.to_string(), Plugin {
            _path: path.to_string(),
            library: lib,
        });
        Ok(())
    }

    pub fn call(&self, plugin: &str, func: &str, args: &str) -> Result<String, String> {
        let p = self.loaded.get(plugin)
            .ok_or_else(|| format!("plugin '{}' not loaded", plugin))?;

        // Try calling e_hello (string -> string)
        let result = unsafe {
            let fn_ptr: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_char> =
                p.library.get(func.as_bytes())
                    .map_err(|e| format!("function '{}' not found in '{}': {}", func, plugin, e))?;
            let c_input = CString::new(args).map_err(|_| "invalid string".to_string())?;
            let c_result = fn_ptr(c_input.as_ptr());
            let result_str = CStr::from_ptr(c_result).to_string_lossy().into_owned();
            // Free the returned string
            let free_fn: libloading::Symbol<unsafe extern "C" fn(*mut c_char)> =
                p.library.get(b"e_free")
                    .map_err(|_| format!("e_free not found in '{}'", plugin))?;
            free_fn(c_result);
            result_str
        };
        Ok(result)
    }

    pub fn has(&self, path: &str) -> bool {
        self.loaded.contains_key(path)
    }
}
