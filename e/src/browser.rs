use headless_chrome::{Browser as HcBrowser, LaunchOptions, Tab};
use std::sync::Arc;

pub struct Browser {
    browser: Option<Arc<HcBrowser>>,
    tab: Option<Arc<Tab>>,
    download_dir: String,
}

impl Browser {
    pub fn new() -> Self {
        Browser { browser: None, tab: None, download_dir: "downloads".into() }
    }

    pub fn start(&mut self, download_dir: &str) {
        self.download_dir = download_dir.to_string();
    }

    fn ensure_browser(&mut self) -> Result<(), String> {
        if self.browser.is_some() { return Ok(()); }

        std::fs::create_dir_all(&self.download_dir).ok();

        let launch_opts = LaunchOptions {
            headless: false,
            sandbox: false,
            window_size: Some((1280, 720)),
            ..LaunchOptions::default()
        };

        let browser = HcBrowser::new(launch_opts)
            .map_err(|e| format!("launch: {}", e))?;
        let tab = browser.new_tab()
            .map_err(|e| format!("new tab: {}", e))?;

        self.browser = Some(Arc::new(browser));
        self.tab = Some(tab);
        Ok(())
    }

    pub fn close(&mut self) {
        self.tab = None;
        self.browser = None;
    }

    pub fn is_running(&self) -> bool {
        self.browser.is_some()
    }

    pub fn open(&mut self, url: &str) -> Result<(), String> {
        self.ensure_browser()?;
        self.tab.as_ref().unwrap().navigate_to(url)
            .map_err(|e| format!("navigate: {}", e))?;
        Ok(())
    }

    pub fn click(&mut self, selector: &str) -> Result<(), String> {
        self.ensure_browser()?;
        let tab = self.tab.as_ref().unwrap();
        let el = tab.wait_for_element(selector)
            .map_err(|e| format!("wait for '{}': {}", selector, e))?;
        el.click()
            .map_err(|e| format!("click '{}': {}", selector, e))?;
        Ok(())
    }

    pub fn find(&mut self, selector: &str) -> Result<(), String> {
        self.ensure_browser()?;
        self.tab.as_ref().unwrap().wait_for_element(selector)
            .map_err(|e| format!("find '{}': {}", selector, e))?;
        Ok(())
    }

    pub fn login(&mut self, user: &str, pass: &str) -> Result<(), String> {
        self.ensure_browser()?;
        let tab = self.tab.as_ref().unwrap();

        // Find and fill username field
        let user_selectors = [
            "input[type='email']", "input[name='email']",
            "input[type='text'][name='username']", "input[name='login']",
            "input[autocomplete='username']", "#username", "#email",
        ];
        let mut logged_in = false;
        for sel in &user_selectors {
            if let Ok(el) = tab.find_element(sel) {
                el.click().map_err(|e| format!("click user: {}", e))?;
                el.type_into(user).map_err(|e| format!("type user: {}", e))?;
                logged_in = true;
                break;
            }
        }
        if !logged_in {
            // Try first text input
            if let Ok(inputs) = tab.find_elements("input[type='text']") {
                if let Some(el) = inputs.first() {
                    el.type_into(user).map_err(|e| format!("type user: {}", e))?;
                }
            }
        }

        // Find and fill password field
        let pass_selectors = [
            "input[type='password']", "#password", "input[name='password']",
            "input[autocomplete='current-password']",
        ];
        let mut pass_found = false;
        for sel in &pass_selectors {
            if let Ok(el) = tab.find_element(sel) {
                el.type_into(pass).map_err(|e| format!("type pass: {}", e))?;
                pass_found = true;
                break;
            }
        }
        if !pass_found {
            return Err("password field not found".into());
        }

        // Click submit
        let submit_selectors = [
            "button[type='submit']", "input[type='submit']",
        ];
        for sel in &submit_selectors {
            if let Ok(el) = tab.find_element(sel) {
                el.click().ok();
                return Ok(());
            }
        }
        // Press Enter as fallback
        tab.press_key("Enter").ok();
        Ok(())
    }

    pub fn wait_download(&mut self) -> Result<String, String> {
        self.ensure_browser()?;
        std::fs::create_dir_all(&self.download_dir)
            .map_err(|e| format!("create download dir: {}", e))?;

        // Poll the download directory for new files
        // headless_chrome downloads to .crdownload while in progress, then renames
        let max_attempts = 60; // 60 * 500ms = 30 seconds timeout
        for _ in 0..max_attempts {
            if let Ok(entries) = std::fs::read_dir(&self.download_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    // Skip directories, temp files (.crdownload), and hidden files
                    if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) { continue; }
                    if name.ends_with(".crdownload") || name.ends_with(".tmp") { continue; }
                    if name.starts_with('.') { continue; }

                    return Ok(path.to_string_lossy().to_string());
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        Err("download timeout (30s) — no file appeared in download directory".into())
    }

    pub fn find_all(&mut self, selector: &str) -> Result<usize, String> {
        self.ensure_browser()?;
        let elements = self.tab.as_ref().unwrap().find_elements(selector)
            .map_err(|e| format!("find all '{}': {}", selector, e))?;
        Ok(elements.len())
    }

    pub fn get_number(&mut self, selector: &str) -> Result<f64, String> {
        self.ensure_browser()?;
        let tab = self.tab.as_ref().unwrap();
        let el = tab.wait_for_element(selector)
            .map_err(|e| format!("find '{}': {}", selector, e))?;
        let text = el.get_inner_text()
            .map_err(|e| format!("text: {}", e))?;
        let re = regex::Regex::new(r"[\d.]+").unwrap();
        if let Some(m) = re.find(&text) {
            m.as_str().parse::<f64>()
                .map_err(|_| format!("cannot parse number from '{}'", &text))
        } else {
            Err(format!("no number found in '{}'", &text))
        }
    }

    pub fn wait_until(&mut self, condition: &str, selector: &str) -> Result<(), String> {
        self.ensure_browser()?;
        let tab = self.tab.as_ref().unwrap();
        match condition {
            "visible" => {
                tab.wait_for_element(selector)
                    .map_err(|e| format!("wait visible '{}': {}", selector, e))?;
                Ok(())
            }
            "hidden" => {
                // Poll for element to disappear
                for _ in 0..100 {
                    if tab.find_element(selector).is_err() {
                        return Ok(());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err("element still visible after timeout".into())
            }
            _ => Err(format!("unknown condition: {}", condition)),
        }
    }
}
