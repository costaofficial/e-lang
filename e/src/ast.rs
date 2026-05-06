#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(i64),
    Str(String),
    List(Vec<Value>),
    Bool(bool),
    Null,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Num(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::List(l) => {
                let items: Vec<String> = l.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Eq, Neq, Gt, Lt, Gte, Lte,
    Add, Sub, Mul, Div,
}

#[derive(Debug, Clone)]
pub enum Condition {
    ItemVisible,
    ItemHidden,
    Compare(String, String, i64), // (target, op, value) — target: "number" | "count"
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Num(i64),
    Str(String),
    Var(String),
    Call(String, Vec<Expr>),
    Bin(Box<Expr>, Op, Box<Expr>),
    Run(String, Option<Box<Expr>>),
    Read(String),
    Ls(String),
    List(Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Slice(Box<Expr>, Box<Expr>, Box<Expr>),
    Method(Box<Expr>, String, Vec<Expr>),
    Len(Box<Expr>),
    Not(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Node {
    TimeNode(Schedule, Vec<Node>, Option<Vec<Node>>),
    ScriptNode(Vec<Node>, Option<Vec<Node>>),
    WithNode(ObjectRef, Option<String>, Vec<Node>, Option<Vec<Node>>),
    RetryNode(i64, Vec<Node>, Option<Vec<Node>>),
    WhenNode(Condition, Vec<Node>, Option<Vec<Node>>),
    WhileNode(Condition, Vec<Node>, Option<Vec<Node>>),
    WatchNode(String, Vec<Node>, Option<Vec<Node>>),
    LetStatement(String, Expr),
    FnDefinition(String, Vec<String>, Vec<Node>),
    ForStatement(String, Expr, Vec<Node>, Option<Vec<Node>>),
    UseStatement(String),
    Action(ActionKind, Vec<Expr>),
    ExprNode(Expr),
}

#[derive(Debug, Clone)]
pub struct Efile {
    pub sys_section: Option<String>,
    pub core_section: Vec<Node>,
    pub ui_section: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Schedule {
    pub kind: String, // "every" or "at"
    pub interval: Option<String>,
    pub time: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ObjectRef {
    File(String),
    Browser,
    Page,
    App(String),
}

#[derive(Debug, Clone)]
pub enum ActionKind {
    Open, Click, Find, FindAll, GetNumber,
    Write, Email, Upload, Login,
    Log, Stop, WaitDownload, WaitUntil(String, String),
    Run, Create,
}
