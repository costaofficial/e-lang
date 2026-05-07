use crate::ast::Value;

pub struct Browser {
    running: bool,
}

impl Browser {
    pub fn new() -> Self {
        Browser { running: false }
    }

    pub fn start(&mut self, _download_dir: &str) {
        self.running = true;
    }

    pub fn close(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn open(&mut self, _url: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn click(&mut self, _selector: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn find(&mut self, _selector: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn login(&mut self, _user: &str, _pass: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn wait_download(&mut self) -> Result<String, String> {
        Ok("downloaded".into())
    }

    pub fn find_all(&mut self, _selector: &str) -> Result<usize, String> {
        Ok(0)
    }

    pub fn get_number(&mut self, _selector: &str) -> Result<Value, String> {
        Ok(Value::Num(0.0))
    }

    pub fn wait_until(&mut self, _condition: &str, _selector: &str) -> Result<(), String> {
        Ok(())
    }
}
