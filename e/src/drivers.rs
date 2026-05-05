use std::process::Command;
use std::fs;
use std::path::Path;

pub trait Driver {
    fn log(&mut self, msg: &str);
    fn run(&mut self, cmd: &str) -> Result<String, String>;
    fn read(&mut self, path: &str) -> Result<String, String>;
    fn ls(&mut self, pattern: &str) -> Result<Vec<String>, String>;
    fn should_stop(&self) -> bool;
    fn set_stop(&mut self, v: bool);
}

pub struct DryDriver {
    stop: bool,
}

impl DryDriver {
    pub fn new() -> Self {
        DryDriver { stop: false }
    }
}

impl Driver for DryDriver {
    fn log(&mut self, msg: &str) {
        println!("  {}", msg);
    }

    fn run(&mut self, cmd: &str) -> Result<String, String> {
        self.log(&format!("⚡ run '{}'", cmd));
        Ok(String::new())
    }

    fn read(&mut self, path: &str) -> Result<String, String> {
        self.log(&format!("📖 read '{}'", path));
        Ok(String::new())
    }

    fn ls(&mut self, pattern: &str) -> Result<Vec<String>, String> {
        self.log(&format!("📂 ls '{}'", pattern));
        Ok(vec![])
    }

    fn should_stop(&self) -> bool { self.stop }
    fn set_stop(&mut self, v: bool) { self.stop = v; }
}

pub struct RealDriver {
    stop: bool,
}

impl RealDriver {
    pub fn new() -> Self {
        RealDriver { stop: false }
    }
}

impl Driver for RealDriver {
    fn log(&mut self, msg: &str) {
        println!("  {}", msg);
    }

    fn run(&mut self, cmd: &str) -> Result<String, String> {
        let output = Command::new("sh").arg("-c").arg(cmd).output()
            .map_err(|e| format!("{}", e))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    fn read(&mut self, path: &str) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| format!("{}", e))
    }

    fn ls(&mut self, pattern: &str) -> Result<Vec<String>, String> {
        let glob_pattern = pattern;
        let mut results = Vec::new();
        if let Some(parent) = Path::new(glob_pattern).parent() {
            if let Ok(entries) = fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if glob_match(glob_pattern, file_name) {
                                results.push(entry.path().to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
        results.sort();
        Ok(results)
    }

    fn should_stop(&self) -> bool { self.stop }
    fn set_stop(&mut self, v: bool) { self.stop = v; }
}

fn glob_match(pattern: &str, name: &str) -> bool {
    if pattern == "*" { return true; }
    if pattern == name { return true; }
    if pattern.starts_with("*.") {
        let ext = &pattern[2..];
        return name.ends_with(ext);
    }
    if pattern.starts_with("*") {
        return name.ends_with(&pattern[1..]);
    }
    if pattern.ends_with("*") {
        return name.starts_with(&pattern[..pattern.len()-1]);
    }
    pattern == name
}
