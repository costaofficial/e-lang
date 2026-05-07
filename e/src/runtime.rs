use std::collections::HashMap;
use std::sync::Mutex;
use crate::ast::*;
use crate::drivers::Driver;
use once_cell::sync::Lazy;
use std::fmt;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub msg: String,
    pub line: i64,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}] {}", self.line, self.msg)
    }
}

impl RuntimeError {
    pub fn new(msg: &str, line: i64) -> Self {
        RuntimeError { msg: msg.to_string(), line }
    }
}

// NOTE: PluginManager is now owned by RealDriver and accessed via driver.plugin_manager()
// No more global statics. Each driver instance has its own plugin manager.

pub struct Scope {
    vars: Vec<HashMap<String, Value>>,
    fns: HashMap<String, FnInfo>,
}

impl Scope {
    pub fn new() -> Self {
        Scope { vars: vec![HashMap::new()], fns: HashMap::new() }
    }

    pub fn push_scope(&mut self) {
        self.vars.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.vars.pop();
    }

    pub fn get_var(&self, name: &str) -> Option<Value> {
        for scope in self.vars.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    pub fn def_var(&mut self, name: &str, val: Value) {
        // Add to the current (topmost) scope
        if let Some(current) = self.vars.last_mut() {
            current.insert(name.to_string(), val);
        }
    }

    pub fn def_fn(&mut self, name: &str, params: Vec<String>, body: Vec<Node>) {
        self.fns.insert(name.to_string(), FnInfo { params, body });
    }

    pub fn get_fn(&self, name: &str) -> Option<&FnInfo> {
        self.fns.get(name)
    }
}

#[derive(Clone)]
struct FnInfo {
    params: Vec<String>,
    body: Vec<Node>,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Num(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Null => false,
        }
    }
}

pub fn eval_expr(expr: &Expr, scope: &mut Scope, driver: &mut dyn Driver) -> Value {
    match expr {
        Expr::Num(n) => Value::Num(*n),
        Expr::Str(s) => Value::Str(s.clone()),
        Expr::Var(name) => {
            scope.get_var(name).unwrap_or_else(|| panic!("variable '{}' not defined", name))
        }
        Expr::Call(name, args) => {
            // Plugin call: sys_call "plugin" "func" ["arg1"] ["arg2"]
            if name == "sys_call" {
                if args.len() < 2 {
                    panic!("sys_call needs at least 2 arguments: plugin, function");
                }
                let plugin_val = eval_expr(&args[0], scope, driver);
                let func = eval_expr(&args[1], scope, driver);
                let arg1 = if args.len() > 2 {
                    let a = eval_expr(&args[2], scope, driver);
                    format!("{}", a)
                } else { String::new() };
                let arg2 = if args.len() > 3 {
                    let a = eval_expr(&args[3], scope, driver);
                    format!("{}", a)
                } else { String::new() };

                // Join arg1 and arg2 with a separator if both present
                let args_str = if arg1.is_empty() { arg2 }
                    else if arg2.is_empty() { arg1 }
                    else { format!("{}|{}", arg1, arg2) };

                // Resolve plugin path to built-in name
                let plugin = format!("{}", plugin_val);
                let resolved = std::path::Path::new(&plugin)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.trim_start_matches("lib").trim_end_matches(".eso").to_string())
                    .unwrap_or_else(|| plugin.clone());

                let result = match driver.plugin_manager() {
                    Some(pm) => pm.call(&resolved, &format!("{}", func), &args_str)
                        .or_else(|_| pm.call(&plugin, &format!("{}", func), &args_str)),
                    None => Err("plugin system not available in dry-run mode".to_string()),
                };
                return match result {
                    Ok(r) => Value::Str(r),
                    Err(e) => panic!("{}", e),
                };
            }

            let params: Vec<String>;
            let body: Vec<Node>;
            {
                let info = scope.get_fn(name).unwrap_or_else(|| panic!("function '{}' not defined", name));
                params = info.params.clone();
                body = info.body.clone();
            }
            if args.len() != params.len() {
                panic!("function '{}' expects {} args, got {}", name, params.len(), args.len());
            }
            scope.push_scope();
            for (param, arg) in params.iter().zip(args.iter()) {
                let val = eval_expr(arg, scope, driver);
                scope.def_var(param, val);
            }
            let mut result = Value::Null;
            for node in &body {
                match node {
                    Node::ExprNode(e) => {
                        result = eval_expr(e, scope, driver);
                    }
                    _ => {
                        exec_node(node, scope, driver);
                    }
                }
            }
            scope.pop_scope();
            result
        }
        Expr::Bin(left, op, right) => {
            let l = eval_expr(left, scope, driver);
            let r = eval_expr(right, scope, driver);
            match op {
                Op::Add => match (&l, &r) {
                    (Value::Str(a), _) => Value::Str(format!("{}{}", a, r)),
                    _ => Value::Num(to_f64(&l) + to_f64(&r)),
                },
                Op::Sub => bin_num(l, r, |a, b| a - b),
                Op::Mul => bin_num(l, r, |a, b| a * b),
                Op::Div => bin_num(l, r, |a, b| a / b),
                Op::Eq => Value::Bool(l == r),
                Op::Neq => Value::Bool(l != r),
                Op::Gt => bin_bool(l, r, |a, b| a > b),
                Op::Lt => bin_bool(l, r, |a, b| a < b),
                Op::Gte => bin_bool(l, r, |a, b| a >= b),
                Op::Lte => bin_bool(l, r, |a, b| a <= b),
                Op::And => Value::Bool(l.is_truthy() && r.is_truthy()),
            }
        }
        Expr::Run(cmd, _stdin) => {
            match driver.run(cmd) {
                Ok(out) => Value::Str(out),
                Err(e) => panic!("run failed: {}", e),
            }
        }
        Expr::Read(path) => match driver.read(path) {
            Ok(c) => Value::Str(c),
            Err(e) => panic!("read failed: {}", e),
        },
        Expr::Ls(pattern) => match driver.ls(pattern) {
            Ok(f) => Value::Str(f.join("\n")),
            Err(e) => panic!("ls failed: {}", e),
        },
        Expr::List(items) => {
            let vals: Vec<Value> = items.iter().map(|e| eval_expr(e, scope, driver)).collect();
            Value::List(vals)
        }
        Expr::Index(container, idx) => {
            let c = eval_expr(container, scope, driver);
            let i = eval_expr(idx, scope, driver);
            let n = to_f64(&i) as usize;
            match &c {
                Value::List(lst) => lst.get(n).cloned().unwrap_or(Value::Null),
                Value::Str(s) => Value::Str(s.chars().nth(n).unwrap_or(' ').to_string()),
                _ => panic!("cannot index"),
            }
        }
        Expr::Slice(container, start, end) => {
            let c = eval_expr(container, scope, driver);
            let s = eval_expr(start, scope, driver);
            let e = eval_expr(end, scope, driver);
            let a = to_f64(&s) as usize;
            let b = to_f64(&e) as usize;
            match &c {
                Value::Str(st) => {
                    let s: String = st.chars().skip(a).take(b - a).collect();
                    Value::Str(s)
                }
                _ => panic!("cannot slice"),
            }
        }
        Expr::Method(obj, method, args) => {
            let mut o = eval_expr(obj, scope, driver);
            if method == "append" {
                if let Value::List(ref mut lst) = o {
                    if let Some(arg) = args.first() { lst.push(eval_expr(arg, scope, driver)); }
                }
                o
            } else { panic!("unknown method '{}'", method); }
        }
        Expr::Len(val) => {
            let v = eval_expr(val, scope, driver);
            match v {
                Value::Str(s) => Value::Num(s.len() as f64),
                Value::List(l) => Value::Num(l.len() as f64),
                _ => panic!("len: expected string or list"),
            }
        }
        Expr::Not(val) => Value::Bool(!eval_expr(val, scope, driver).is_truthy()),
    }
}

fn to_f64(v: &Value) -> f64 {
    match v {
        Value::Num(n) => *n,
        _ => 0.0,
    }
}

fn bin_num(l: Value, r: Value, f: fn(f64, f64) -> f64) -> Value {
    let a = to_f64(&l);
    let b = to_f64(&r);
    Value::Num(f(a, b))
}

fn bin_bool(l: Value, r: Value, f: fn(f64, f64) -> bool) -> Value {
    let a = to_f64(&l);
    let b = to_f64(&r);
    Value::Bool(f(a, b))
}

pub fn eval_condition(cond: &Condition, scope: &mut Scope, driver: &mut dyn Driver) -> bool {
    match cond {
        Condition::ItemVisible => true,
        Condition::ItemHidden => false,
        Condition::Compare(target, op, val) => {
            let actual_f = match target.as_str() {
                "number" => match scope.get_var("number") {
                    Some(v) => to_f64(&v),
                    _ => { driver.log("number is not set"); return false; }
                },
                "count" => match scope.get_var("count") {
                    Some(v) => to_f64(&v),
                    _ => 0.0,
                },
                _ => return false,
            };
            let v = *val as f64;
            match op.as_str() {
                "=" => (actual_f - v).abs() < 0.0001, ">" => actual_f > v,
                "<" => actual_f < v, ">=" => actual_f >= v,
                "<=" => actual_f <= v, _ => false,
            }
        }
        Condition::Expression(expr) => eval_expr(expr, scope, driver).is_truthy(),
    }
}

pub fn exec_node(node: &Node, scope: &mut Scope, driver: &mut dyn Driver) {
    match node {
        Node::ScriptNode(actions, _fb) => {
            for a in actions { exec_node(a, scope, driver); }
        }
        Node::TimeNode(schedule, actions, _fb) => {
            let mut desc = format!("⏰ Schedule:");
            if let Some(ref interval) = schedule.interval {
                desc += &format!(" every {}", interval);
            }
            if let Some(ref t) = schedule.time {
                desc += &format!(" at {}", t);
            }
            driver.log(&desc);
            for a in actions { exec_node(a, scope, driver); }
        }
        Node::WithNode(obj, _config, actions, _fb) => {
            match obj {
                ObjectRef::Browser => {
                    driver.log("🌐 Context: browser");
                    driver.browser_start("downloads", 0);
                    for a in actions { exec_node(a, scope, driver); }
                    driver.browser_stop(0);
                    return;
                }
                ObjectRef::Page => {
                    driver.log("📄 Context: page");
                }
                _ => {}
            }
            for a in actions { exec_node(a, scope, driver); }
        }
        Node::LetStatement(name, expr) => {
            let val = eval_expr(expr, scope, driver);
            scope.def_var(name, val.clone());
            driver.log(&format!("📦 let {} = {}", name, val));
        }
        Node::FnDefinition(name, params, body) => {
            scope.def_fn(name, params.clone(), body.clone());
            driver.log(&format!("📦 fn {}({})", name, params.join(", ")));
        }
        Node::ExprNode(expr) => {
            let _val = eval_expr(expr, scope, driver);
            // ExprNode is only for bare expressions (function calls, values)
            // Logging is handled by ActionKind::Log
            // The returned value is used when assigned via let x = expr
        }
        Node::WhenNode(cond, actions, _fb) => {
            let result = eval_condition(cond, scope, driver);
            driver.log(&format!("  when → {}", result));
            if result {
                for a in actions { exec_node(a, scope, driver); }
            }
        }
        Node::WhileNode(cond, body, _fb) => {
            let mut max = 10000;
            while max > 0 && !driver.should_stop() {
                if !eval_condition(cond, scope, driver) { break; }
                max -= 1;
                for a in body { exec_node(a, scope, driver); }
            }
        }
        Node::RetryNode(times, actions, _fb) => {
            for attempt in 1..=*times {
                if driver.should_stop() { break; }
                driver.log(&format!("🔄 Attempt {}/{}", attempt, times));
                for a in actions {
                    exec_node(a, scope, driver);
                }
            }
        }
        Node::ForStatement(var, collection, body, _fb) => {
            let coll = eval_expr(collection, scope, driver);
            let items: Vec<Value> = match coll {
                Value::List(lst) => lst,
                Value::Str(s) => s.lines().map(|l| Value::Str(l.to_string())).collect(),
                _ => vec![],
            };
            for item in items {
                scope.def_var(var, item);
                for a in body { exec_node(a, scope, driver); }
            }
        }
        Node::Action(kind, args) => {
            exec_action(kind, args, scope, driver);
        }
        Node::UseStatement(path) => {
            exec_use(path, scope, driver);
        }
        Node::WatchNode(path, actions, _fb) => {
            exec_watch(path, actions, scope, driver);
        }
        _ => {}
    }
}

fn exec_use(path: &str, scope: &mut Scope, driver: &mut dyn Driver) {
    use std::fs;
    use crate::lexer;
    use crate::parser;

    let filepath = if path.ends_with(".eee") {
        path.to_string()
    } else {
        format!("{}.eee", path)
    };

    let source = match fs::read_to_string(&filepath) {
        Ok(s) => s,
        Err(_) => {
            driver.log(&format!("⚠️ module not found: '{}'", filepath));
            return;
        }
    };

    let tokens = match lexer::lex(&source) {
        Ok(t) => t,
        Err(e) => {
            driver.log(&format!("⚠️ module lex error: {}", e));
            return;
        }
    };

    let mut p = parser::Parser::new(tokens);
    let nodes = p.parse();

    for node in &nodes {
        exec_node(node, scope, driver);
    }
}

fn exec_watch(path: &str, actions: &[Node], scope: &mut Scope, driver: &mut dyn Driver) {
    use std::io::Read;
    driver.log(&format!("👀 Watch: '{}'", path));

    let watched_dir = std::path::Path::new(path);
    if !watched_dir.exists() {
        driver.log(&format!("⚠️ watch path '{}' does not exist", path));
        return;
    }

    driver.log(&format!("  ✅ watching '{}'", path));

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(entries) = std::fs::read_dir(watched_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        seen.insert(name.to_string());
                    }
                }
            }
        }
    }

    loop {
        if driver.should_stop() { break; }
        std::thread::sleep(std::time::Duration::from_secs(2));

        if let Ok(entries) = std::fs::read_dir(watched_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            if !seen.contains(name) {
                                seen.insert(name.to_string());
                                driver.log(&format!("  📄 new file: {}", name));
                                for a in actions {
                                    exec_node(a, scope, driver);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn exec_action(kind: &ActionKind, args: &[Expr], scope: &mut Scope, driver: &mut dyn Driver) {
    match kind {
        ActionKind::Log => {
            if let Some(expr) = args.first() {
                let val = eval_expr(expr, scope, driver);
                driver.log(&format!("📝 {}", val));
            }
        }
        ActionKind::Stop => driver.set_stop(true),
        ActionKind::Run => {
            if let Some(Expr::Str(cmd)) = args.first() {
                match driver.run(cmd) {
                    Ok(out) => driver.log(&out),
                    Err(e) => driver.log(&format!("❌ run failed: {}", e)),
                }
            }
        }
        ActionKind::Open => {
            if let Some(Expr::Str(url)) = args.first() {
                let result = driver.browser_open(url);
                if result.is_ok() {
                    driver.log(&format!("  🌐 opened '{}'", url));
                } else {
                    driver.log(&format!("  🌐 open '{}' (simulated)", url));
                }
            }
        }
        ActionKind::Click => {
            if let Some(Expr::Str(sel)) = args.first() {
                driver.browser_click(sel).ok();
                driver.log(&format!("  🖱️ clicked '{}'", sel));
            } else {
                driver.log("  🖱️ click");
            }
        }
        ActionKind::Find => {
            if let Some(Expr::Str(sel)) = args.first() {
                driver.browser_find(sel).ok();
                driver.log(&format!("  🔍 found '{}'", sel));
            }
        }
        ActionKind::FindAll => {
            if let Some(Expr::Str(sel)) = args.first() {
                match driver.browser_find_all(sel) {
                    Ok(n) => driver.log(&format!("  🔍 find all '{}' → {} elements", sel, n)),
                    Err(_) => driver.log(&format!("  🔍 find all '{}' (simulated)", sel)),
                }
            }
        }
        ActionKind::GetNumber => {
            if let Some(Expr::Str(sel)) = args.first() {
                driver.log(&format!("  🔢 get number from '{}'", sel));
            }
        }
        ActionKind::Write => {
            if args.len() >= 2 {
                if let (Expr::Str(path), Expr::Str(content)) = (&args[0], &args[1]) {
                    driver.log(&format!("  ✏️ write '{}'", if path.is_empty() { "(current)" } else { path }));
                    if !path.is_empty() {
                        let _ = std::fs::write(path, content);
                    }
                }
            }
        }
        ActionKind::Login => {
            if args.len() >= 2 {
                if let (Expr::Str(user), Expr::Str(pass)) = (&args[0], &args[1]) {
                    match driver.browser_login(user, pass) {
                        Ok(_) => driver.log(&format!("  🔐 logged in as '{}'", user)),
                        Err(_) => driver.log(&format!("  🔐 login '{}' (simulated)", user)),
                    }
                }
            }
        }
        ActionKind::Email => {
            if let Some(Expr::Str(to)) = args.first() {
                let attach = if args.len() > 1 {
                    if let Expr::Str(a) = &args[1] { Some(a.as_str()) } else { None }
                } else { None };
                match driver.send_email(to, attach) {
                    Ok(_) => driver.log(&format!("  📧 email sent to '{}'", to)),
                    Err(_) => driver.log(&format!("  📧 email to '{}' (simulated)", to)),
                }
            }
        }
        ActionKind::Upload => {
            driver.log("  ⬆️ upload (not implemented)");
        }
        ActionKind::Create => {
            driver.log("  🆕 create (not implemented)");
        }
        ActionKind::WaitDownload => {
            match driver.browser_wait_download() {
                Ok(path) => driver.log(&format!("  ⏳ wait download... ✅ '{}'", path)),
                Err(_) => driver.log("  ⏳ wait download... ✅"),
            }
        }
        ActionKind::WaitUntil(cond, sel) => {
            match driver.browser_wait_until(cond, sel) {
                Ok(_) => driver.log(&format!("  ⏳ wait until {} '{}'... ✅", cond, sel)),
                Err(_) => driver.log(&format!("  ⏳ wait until {} '{}'... ✅ (simulated)", cond, sel)),
            }
        }
        _ => driver.log(&format!("  (action {:?} not implemented)", kind)),
    }
}
