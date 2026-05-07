use std::collections::HashMap;

type PluginFn = fn(&str) -> String;

pub struct PluginManager {
    plugins: HashMap<String, HashMap<String, PluginFn>>,
    loaded: HashMap<String, bool>,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager { plugins: HashMap::new(), loaded: HashMap::new() }
    }

    pub fn register(&mut self, name: &str, functions: HashMap<String, PluginFn>) {
        self.plugins.insert(name.to_string(), functions);
        self.loaded.insert(name.to_string(), true);
    }

    pub fn load(&mut self, path: &str) -> Result<(), String> {
        if self.loaded.contains_key(path) {
            return Ok(());
        }
        Err(format!("plugin '{}' not built-in. Available: json, fs, db", path))
    }

    pub fn call(&self, plugin: &str, func: &str, args: &str) -> Result<String, String> {
        let p = self.plugins.get(plugin)
            .ok_or_else(|| format!("plugin '{}' not registered", plugin))?;
        let f = p.get(func)
            .ok_or_else(|| format!("function '{}' not found in '{}'", func, plugin))?;
        Ok(f(args))
    }

    pub fn has(&self, path: &str) -> bool {
        self.loaded.contains_key(path)
    }

    pub fn register_std(&mut self) {
        // Register standard library plugins as built-in functions
        let mut http_fns: HashMap<String, PluginFn> = HashMap::new();
        http_fns.insert("e_get".to_string(), sys_http_get);
        http_fns.insert("e_post".to_string(), sys_http_post);
        self.register("http", http_fns);

        let mut json_fns: HashMap<String, PluginFn> = HashMap::new();
        json_fns.insert("e_parse".to_string(), sys_json_parse);
        json_fns.insert("e_stringify".to_string(), sys_json_stringify);
        self.register("json", json_fns);

        let mut fs_fns: HashMap<String, PluginFn> = HashMap::new();
        fs_fns.insert("e_exists".to_string(), sys_fs_exists);
        fs_fns.insert("e_size".to_string(), sys_fs_size);
        fs_fns.insert("e_copy".to_string(), sys_fs_copy);
        fs_fns.insert("e_delete".to_string(), sys_fs_delete);
        self.register("fs", fs_fns);

        let mut db_fns: HashMap<String, PluginFn> = HashMap::new();
        db_fns.insert("e_open".to_string(), sys_db_open);
        db_fns.insert("e_query".to_string(), sys_db_query);
        self.register("db", db_fns);
    }
}

fn json_result(s: &str) -> String {
    format!("{{\"ok\": true, \"result\": {}}}", s)
}

// JSON plugin
fn sys_json_parse(input: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(input) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_else(|_| input.to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

fn sys_json_stringify(input: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(input) {
        Ok(v) => v.to_string(),
        Err(_) => format!("\"{}\"", input.replace('"', "\\\"")),
    }
}

// FS plugin
fn sys_fs_exists(path: &str) -> String {
    if std::path::Path::new(path).exists() { "true".to_string() } else { "false".to_string() }
}

fn sys_fs_size(path: &str) -> String {
    std::fs::metadata(path).map(|m| m.len().to_string()).unwrap_or_else(|_| "0".to_string())
}

fn sys_fs_copy(args: &str) -> String {
    let parts: Vec<&str> = args.splitn(2, '|').collect();
    if parts.len() < 2 { return "{{\"error\": \"need src|dst\"}}".to_string(); }
    match std::fs::copy(parts[0], parts[1]) {
        Ok(n) => format!("{{\"ok\": true, \"bytes\": {}}}", n),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

fn sys_fs_delete(path: &str) -> String {
    match std::fs::remove_file(path) {
        Ok(_) => "true".to_string(),
        Err(e) => format!("false: {}", e),
    }
}

// DB plugin (file-backed JSON)
use std::sync::Mutex;
use std::path::Path;

static DB_FILE: &str = "e_data.json";
static DB: once_cell::sync::Lazy<Mutex<HashMap<String, Vec<HashMap<String, String>>>>> =
    once_cell::sync::Lazy::new(|| {
        let data = std::fs::read_to_string(DB_FILE)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Mutex::new(data)
    });

fn save_db() {
    let db = DB.lock().unwrap();
    if let Ok(json) = serde_json::to_string(&*db) {
        let _ = std::fs::write(DB_FILE, &json);
    }
}

fn sys_db_open(path: &str) -> String {
    if !path.is_empty() {
        // Load custom database file
        let data = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let mut db = DB.lock().unwrap();
        *db = data;
    }
    "{\"ok\": true}".to_string()
}

fn sys_db_query(input: &str) -> String {
    let parts: Vec<&str> = input.splitn(2, '|').collect();
    let _table = parts.get(0).unwrap_or(&"");
    let sql = parts.get(1).unwrap_or(&"");
    let sql_upper = sql.trim().to_uppercase();
    let result;

    {
        let mut db = DB.lock().unwrap();

        if sql_upper.starts_with("CREATE") {
            let table_name = sql.split_whitespace().nth(2).unwrap_or("table");
            db.entry(table_name.to_string()).or_insert_with(Vec::new);
            result = format!("{{\"ok\": true, \"table\": \"{}\"}}", table_name);
        } else if sql_upper.starts_with("INSERT") {
            let table_name = sql.split_whitespace().nth(2).unwrap_or("table");
            let rows = db.entry(table_name.to_string()).or_insert_with(Vec::new);
            if let Some(values_start) = sql.find("VALUES") {
                let values_str = &sql[values_start + 6..].trim().trim_matches(&['(', ')'][..]);
                let mut row = HashMap::new();
                for (i, val) in values_str.split(',').enumerate() {
                    let v = val.trim().trim_matches('\'');
                    row.insert(format!("col{}", i), v.to_string());
                }
                rows.push(row);
            }
            result = format!("{{\"ok\": true, \"table\": \"{}\"}}", table_name);
        } else if sql_upper.starts_with("SELECT") {
            let table_name = sql.split_whitespace().nth(3).unwrap_or("table");
            let rows = db.get(table_name).cloned().unwrap_or_default();
            result = serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into());
        } else {
            result = "null".to_string();
        }
    }

    save_db();
    result
}

// HTTP plugin
fn sys_http_get(url: &str) -> String {
    match ureq::get(url).call() {
        Ok(resp) => match resp.into_string() {
            Ok(body) => serde_json::to_string(&body).unwrap_or_else(|_| format!("\"{}\"", body)),
            Err(e) => format!("{{\"error\": \"read failed: {}\"}}", e),
        },
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

fn sys_http_post(args: &str) -> String {
    let parts: Vec<&str> = args.splitn(2, '|').collect();
    if parts.len() < 2 { return "{{\"error\": \"need url|body\"}}".to_string(); }
    match ureq::post(parts[0]).send_string(parts[1]) {
        Ok(resp) => match resp.into_string() {
            Ok(body) => body,
            Err(e) => format!("{{\"error\": \"read failed: {}\"}}", e),
        },
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}


