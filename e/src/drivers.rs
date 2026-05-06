use std::process::Command;
use std::fs;
use std::path::Path;
use crate::browser::Browser;
use crate::email::Mailer;

pub trait Driver {
    fn log(&mut self, msg: &str);
    fn run(&mut self, cmd: &str) -> Result<String, String>;
    fn read(&mut self, path: &str) -> Result<String, String>;
    fn ls(&mut self, pattern: &str) -> Result<Vec<String>, String>;
    fn should_stop(&self) -> bool;
    fn set_stop(&mut self, v: bool);

    // Browser
    fn browser_start(&mut self, _download_dir: &str, _line: i64) {}
    fn browser_stop(&mut self, _line: i64) {}
    fn browser_open(&mut self, _url: &str) -> Result<(), String> { Ok(()) }
    fn browser_click(&mut self, _sel: &str) -> Result<(), String> { Ok(()) }
    fn browser_find(&mut self, _sel: &str) -> Result<(), String> { Ok(()) }
    fn browser_login(&mut self, _user: &str, _pass: &str) -> Result<(), String> { Ok(()) }
    fn browser_wait_download(&mut self) -> Result<String, String> { Ok("".into()) }
    fn browser_find_all(&mut self, _sel: &str) -> Result<usize, String> { Ok(0) }
    fn browser_get_number(&mut self, _sel: &str) -> Result<i64, String> { Ok(0) }
    fn browser_wait_until(&mut self, _cond: &str, _sel: &str) -> Result<(), String> { Ok(()) }

    // Email
    fn send_email(&mut self, _to: &str, _attachment: Option<&str>) -> Result<(), String> { Ok(()) }
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
    fn log(&mut self, msg: &str) { println!("  {}", msg); }
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
    browser: Option<Browser>,
    mailer: Option<Mailer>,
}

impl RealDriver {
    pub fn new() -> Self {
        RealDriver { stop: false, browser: None, mailer: None }
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
        let mut results = Vec::new();
        let path = Path::new(pattern);
        let parent = path.parent().unwrap_or(Path::new("."));
        // Extract the filename pattern from the full path pattern
        let file_pattern = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("*");
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    if let Some(name) = entry.file_name().to_str() {
                        if glob_match(file_pattern, name) {
                            results.push(entry.path().to_string_lossy().to_string());
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

    // Browser
    fn browser_start(&mut self, download_dir: &str, _line: i64) {
        let mut b = Browser::new();
        b.start(download_dir);
        self.browser = Some(b);
    }

    fn browser_stop(&mut self, _line: i64) {
        if let Some(ref mut b) = self.browser {
            b.close();
        }
        self.browser = None;
    }

    fn browser_open(&mut self, url: &str) -> Result<(), String> {
        if let Some(ref mut b) = self.browser { b.open(url) } else { Ok(()) }
    }

    fn browser_click(&mut self, sel: &str) -> Result<(), String> {
        if let Some(ref mut b) = self.browser { b.click(sel) } else { Ok(()) }
    }

    fn browser_find(&mut self, sel: &str) -> Result<(), String> {
        if let Some(ref mut b) = self.browser { b.find(sel) } else { Ok(()) }
    }

    fn browser_login(&mut self, user: &str, pass: &str) -> Result<(), String> {
        if let Some(ref mut b) = self.browser { b.login(user, pass) } else { Ok(()) }
    }

    fn browser_wait_download(&mut self) -> Result<String, String> {
        if let Some(ref mut b) = self.browser { b.wait_download() } else { Ok("".into()) }
    }

    fn browser_find_all(&mut self, sel: &str) -> Result<usize, String> {
        if let Some(ref mut b) = self.browser { b.find_all(sel) } else { Ok(0) }
    }

    fn browser_get_number(&mut self, sel: &str) -> Result<i64, String> {
        if let Some(ref mut b) = self.browser { b.get_number(sel).map(|v| match v { _ => 0 }) } else { Ok(0) }
    }

    fn browser_wait_until(&mut self, cond: &str, sel: &str) -> Result<(), String> {
        if let Some(ref mut b) = self.browser { b.wait_until(cond, sel) } else { Ok(()) }
    }

    // Email
    fn send_email(&mut self, to: &str, _attachment: Option<&str>) -> Result<(), String> {
        if self.mailer.is_none() {
            self.mailer = Some(Mailer::new());
        }
        if let Some(ref m) = self.mailer {
            m.send(to, _attachment)
        } else {
            Ok(())
        }
    }
}

fn glob_match(pattern: &str, name: &str) -> bool {
    if pattern == "*" { return true; }
    if pattern == name { return true; }
    if pattern.starts_with("*.") {
        return name.ends_with(&pattern[2..]);
    }
    if pattern.starts_with("*") {
        return name.ends_with(&pattern[1..]);
    }
    if pattern.ends_with("*") {
        return name.starts_with(&pattern[..pattern.len()-1]);
    }
    pattern == name
}
