use std::process::Command;

pub struct Browser {
    running: bool,
    page_connected: bool,
    _phantom: (),
}

impl Browser {
    pub fn new() -> Self {
        Browser { running: false, page_connected: false, _phantom: () }
    }

    pub fn start(&mut self, _download_dir: &str) {
        self.running = true;
    }

    pub fn close(&mut self) {
        self.running = false;
        self.page_connected = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn open(&mut self, url: &str) -> Result<(), String> {
        let browsers = ["google-chrome", "google-chrome-stable", "chromium-browser", "chromium"];
        let mut launched = false;
        for browser in &browsers {
            if let Ok(_) = Command::new(browser).arg("--new-window").arg(url).spawn() {
                self.running = true;
                self.page_connected = true;
                launched = true;
                break;
            }
        }
        if !launched {
            if let Ok(_) = Command::new("xdg-open").arg(url).spawn() {
                self.running = true;
                self.page_connected = true;
                launched = true;
            }
        }
        if launched { Ok(()) }
        else { Err("no browser found (tried: google-chrome, chromium-browser, xdg-open)".into()) }
    }

    pub fn click(&mut self, _selector: &str) -> Result<(), String> {
        Err("click needs chromiumoxide".into())
    }

    pub fn find(&mut self, _selector: &str) -> Result<(), String> {
        Err("find needs chromiumoxide".into())
    }

    pub fn login(&mut self, _user: &str, _pass: &str) -> Result<(), String> {
        Err("login needs chromiumoxide".into())
    }

    pub fn wait_download(&mut self) -> Result<String, String> {
        Err("wait download needs chromiumoxide".into())
    }

    pub fn find_all(&mut self, _selector: &str) -> Result<usize, String> {
        Err("find all needs chromiumoxide".into())
    }

    pub fn get_number(&mut self, _selector: &str) -> Result<f64, String> {
        Err("get number needs chromiumoxide".into())
    }

    pub fn wait_until(&mut self, _condition: &str, _selector: &str) -> Result<(), String> {
        Err("wait until needs chromiumoxide".into())
    }
}
