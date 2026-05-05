use crate::ast::*;
use crate::lexer::*;

macro_rules! is_kw {
    ($t:expr, $kw:expr) => {{
        let k = &$t.kind;
        token_name(k) == $kw
    }};
}

fn token_name(k: &TokenKind) -> String {
    match k {
        TokenKind::Ident(s) => s.clone(),
        TokenKind::Number(_) => "number".into(),
        TokenKind::String(_) => "string".into(),
        TokenKind::LBrace => "{".into(),
        TokenKind::RBrace => "}".into(),
        TokenKind::LBracket => "[".into(),
        TokenKind::RBracket => "]".into(),
        TokenKind::Op(s) => s.clone(),
        TokenKind::DotDot => "..".into(),
        TokenKind::Time => "time".into(),
        TokenKind::Every => "every".into(),
        TokenKind::At => "at".into(),
        TokenKind::Do => "do".into(),
        TokenKind::Done => "done".into(),
        TokenKind::With => "with".into(),
        TokenKind::Or => "or".into(),
        TokenKind::Retry => "retry".into(),
        TokenKind::Times => "times".into(),
        TokenKind::Wait => "wait".into(),
        TokenKind::Until => "until".into(),
        TokenKind::Watch => "watch".into(),
        TokenKind::Login => "login".into(),
        TokenKind::Stop => "stop".into(),
        TokenKind::Write => "write".into(),
        TokenKind::Email => "email".into(),
        TokenKind::Upload => "upload".into(),
        TokenKind::Click => "click".into(),
        TokenKind::Find => "find".into(),
        TokenKind::Log => "log".into(),
        TokenKind::File => "file".into(),
        TokenKind::Browser => "browser".into(),
        TokenKind::Page => "page".into(),
        TokenKind::App => "app".into(),
        TokenKind::Visible => "visible".into(),
        TokenKind::Hidden => "hidden".into(),
        TokenKind::Download => "download".into(),
        TokenKind::To => "to".into(),
        TokenKind::Timeout => "timeout".into(),
        TokenKind::When => "when".into(),
        TokenKind::All => "all".into(),
        TokenKind::Get => "get".into(),
        TokenKind::From => "from".into(),
        TokenKind::NumKw => "number".into(),
        TokenKind::Item => "item".into(),
        TokenKind::Count => "count".into(),
        TokenKind::Let => "let".into(),
        TokenKind::Fn => "fn".into(),
        TokenKind::Run => "run".into(),
        TokenKind::Read => "read".into(),
        TokenKind::Ls => "ls".into(),
        TokenKind::For => "for".into(),
        TokenKind::In => "in".into(),
        TokenKind::Use => "use".into(),
        TokenKind::Append => "append".into(),
        TokenKind::While => "while".into(),
        TokenKind::Len => "len".into(),
        TokenKind::Not => "not".into(),
        TokenKind::Hour => "hour".into(),
        TokenKind::Day => "day".into(),
        TokenKind::Minute => "minute".into(),
        TokenKind::S => "s".into(),
        TokenKind::Ms => "ms".into(),
        TokenKind::Eof => "eof".into(),
        TokenKind::Newline => "newline".into(),
        TokenKind::Comma => ",".into(),
        TokenKind::Colon => ":".into(),
    }
}

fn is_expr_start(k: &TokenKind) -> bool {
    matches!(k,
        TokenKind::Number(_) | TokenKind::String(_) | TokenKind::Ident(_)
        | TokenKind::LBracket | TokenKind::Run | TokenKind::Read
        | TokenKind::Ls | TokenKind::Len | TokenKind::Not
        | TokenKind::Item | TokenKind::Count | TokenKind::NumKw
    ) || matches!(k, TokenKind::Op(s) if s == "-" || s == "(")
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token { &self.tokens[self.pos] }
    fn pop(&mut self) -> Token { let t = self.peek().clone(); self.pos += 1; t }

    fn expect_ident(&mut self, expected: &str) -> Token {
        let t = self.pop();
        let name = token_name(&t.kind);
        if name != expected {
            panic!("line {}: expected '{}', got '{}'", t.line, expected, name);
        }
        t
    }

    fn expect_token(&mut self, expected: &str) -> Token {
        let t = self.pop();
        let name = token_name(&t.kind);
        if name != expected {
            panic!("line {}: expected '{}', got '{}'", t.line, expected, name);
        }
        t
    }

    fn skip_newlines(&mut self) {
        while self.pos < self.tokens.len() && matches!(self.peek().kind, TokenKind::Newline) {
            self.pos += 1;
        }
    }

    fn peek_next(&self) -> TokenKind {
        let mut p = self.pos + 1;
        while p < self.tokens.len() && matches!(self.tokens[p].kind, TokenKind::Newline) { p += 1; }
        if p < self.tokens.len() { self.tokens[p].kind.clone() } else { TokenKind::Eof }
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek().kind, TokenKind::Eof) {
            nodes.push(self.parse_statement_block());
            self.skip_newlines();
        }
        nodes
    }

    fn parse_statement_block(&mut self) -> Node {
        let k = self.peek().kind.clone();
        if matches!(k, TokenKind::Time) { return self.parse_time_block(); }
        if matches!(k, TokenKind::Do) { return self.parse_script_block(); }
        if matches!(k, TokenKind::Fn) { return self.parse_fn(); }
        if matches!(k, TokenKind::Let) { return self.parse_let(); }
        if matches!(k, TokenKind::Use) { return self.parse_use(); }
        panic!("line {}: expected statement, got {:?}", self.peek().line, k);
    }

    fn parse_time_block(&mut self) -> Node {
        let _line = self.pop().line;
        let schedule = self.parse_schedule();
        self.expect_ident("do");
        let actions = self.parse_actions();
        self.expect_ident("done");
        Node::TimeNode(schedule, actions, self.parse_optional_fallback())
    }

    fn parse_schedule(&mut self) -> Schedule {
        match self.peek().kind.clone() {
            TokenKind::Every => {
                self.pop();
                let interval = token_name(&self.pop().kind);
                let time = if matches!(self.peek().kind, TokenKind::At) {
                    self.pop(); Some(self.parse_time())
                } else { None };
                Schedule { kind: "every".into(), interval: Some(interval), time }
            }
            TokenKind::At => {
                self.pop();
                Schedule { kind: "at".into(), interval: None, time: Some(self.parse_time()) }
            }
            _ => panic!("line {}: expected 'every' or 'at'", self.peek().line),
        }
    }

    fn parse_time(&mut self) -> String {
        let h = match self.pop().kind { TokenKind::Number(n) => n, _ => panic!("expected number") };
        if matches!(self.peek().kind, TokenKind::Colon) {
            self.pop();
            let m = match self.pop().kind { TokenKind::Number(n) => n, _ => panic!("expected number") };
            format!("{}:{}", h, m)
        } else { format!("{}:00", h) }
    }

    fn parse_script_block(&mut self) -> Node {
        let _line = self.pop().line;
        let actions = self.parse_actions();
        self.expect_ident("done");
        Node::ScriptNode(actions, self.parse_optional_fallback())
    }

    fn parse_actions(&mut self) -> Vec<Node> {
        let mut a = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek().kind, TokenKind::Done | TokenKind::Eof) {
            if matches!(self.peek().kind, TokenKind::Or) { break; }
            a.push(self.parse_statement());
            self.skip_newlines();
        }
        a
    }

    fn parse_statement(&mut self) -> Node {
        let mut core = self.parse_core_statement();
        self.skip_newlines();
        if matches!(self.peek().kind, TokenKind::Or) {
            let fb = self.parse_fallback();
            set_fallback(&mut core, fb);
        }
        core
    }

    fn parse_core_statement(&mut self) -> Node {
        let k = self.peek().kind.clone();
        if matches!(k, TokenKind::With) { return self.parse_with_block(); }
        if matches!(k, TokenKind::Retry) { return self.parse_retry_block(); }
        if matches!(k, TokenKind::Watch) { return self.parse_watch_block(); }
        if matches!(k, TokenKind::Wait) { return self.parse_wait(); }
        if matches!(k, TokenKind::Login) { return self.parse_login(); }
        if matches!(k, TokenKind::Stop) { return self.parse_stop(); }
        if matches!(k, TokenKind::Write) { return self.parse_write(); }
        if matches!(k, TokenKind::Email) || matches!(k, TokenKind::Upload) { return self.parse_transfer(); }
        if matches!(k, TokenKind::Find) && matches!(self.peek_next(), TokenKind::All) { return self.parse_find_all(); }
        if matches!(k, TokenKind::Click) || matches!(k, TokenKind::Find) { return self.parse_ui_action(); }
        if matches!(k, TokenKind::Log) { return self.parse_log(); }
        if matches!(k, TokenKind::When) { return self.parse_when(); }
        if matches!(k, TokenKind::While) { return self.parse_while(); }
        if matches!(k, TokenKind::Get) { return self.parse_get_number(); }
        if matches!(k, TokenKind::Let) { return self.parse_let(); }
        if matches!(k, TokenKind::Fn) { return self.parse_fn(); }
        if matches!(k, TokenKind::For) { return self.parse_for(); }
        if matches!(k, TokenKind::Use) { return self.parse_use(); }
        // Simple actions: open, create — check by ident name
        if matches!(&k, TokenKind::Ident(s) if s == "open" || s == "create") {
            let name = match &k { TokenKind::Ident(s) => s.clone(), _ => unreachable!() };
            self.pop();
            return if name == "open" {
                Node::Action(ActionKind::Open, vec![Expr::Str(match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") })])
            } else {
                Node::Action(ActionKind::Create, vec![])
            };
        }
        if is_expr_start(&k) || matches!(k, TokenKind::Op(ref s) if s == "-" || s == "(") {
            return self.parse_expr_stmt();
        }
        panic!("line {}: unknown action {:?}", self.peek().line, k);
    }

    fn parse_optional_fallback(&mut self) -> Option<Vec<Node>> {
        self.skip_newlines();
        if matches!(self.peek().kind, TokenKind::Or) { Some(self.parse_fallback()) } else { None }
    }

    fn parse_fallback(&mut self) -> Vec<Node> {
        self.pop();
        self.skip_newlines();
        if matches!(self.peek().kind, TokenKind::Do) {
            self.pop();
            let a = self.parse_actions();
            self.expect_ident("done");
            a
        } else { vec![self.parse_core_statement()] }
    }

    fn parse_with_block(&mut self) -> Node {
        let _line = self.pop().line;
        let obj = self.parse_object();
        let config = if matches!(self.peek().kind, TokenKind::LBrace) {
            self.pop(); self.expect_ident("timeout"); self.expect_ident(":");
            let n = match self.pop().kind { TokenKind::Number(n) => n, _ => panic!("expected number") };
            let s = token_name(&self.peek().kind);
            let cfg = if s == "ms" { format!("{}ms", n) } else { format!("{}s", n) };
            self.pop(); self.expect_ident("}"); Some(cfg)
        } else { None };
        self.expect_ident("do");
        let actions = self.parse_actions();
        self.expect_ident("done");
        Node::WithNode(obj, config, actions, None)
    }

    fn parse_object(&mut self) -> ObjectRef {
        match &self.pop().kind {
            TokenKind::File => { ObjectRef::File(match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") }) }
            TokenKind::Browser => ObjectRef::Browser,
            TokenKind::Page => ObjectRef::Page,
            TokenKind::App => { ObjectRef::App(match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") }) }
            _ => panic!("line {}: unknown object", self.peek().line),
        }
    }

    fn parse_retry_block(&mut self) -> Node {
        let _line = self.pop().line;
        let n = match self.pop().kind { TokenKind::Number(n) => n, _ => panic!("expected number") };
        self.expect_ident("times"); self.expect_ident("do");
        let a = self.parse_actions(); self.expect_ident("done");
        Node::RetryNode(n, a, None)
    }

    fn parse_watch_block(&mut self) -> Node {
        let _line = self.pop().line;
        let p = match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") };
        self.expect_ident("do"); let a = self.parse_actions(); self.expect_ident("done");
        Node::WatchNode(p, a, None)
    }

    fn parse_wait(&mut self) -> Node {
        let _line = self.pop().line;
        if matches!(self.peek().kind, TokenKind::Download) { self.pop(); return Node::Action(ActionKind::WaitDownload, vec![]); }
        self.expect_ident("until");
        let cond = token_name(&self.pop().kind);
        let sel = match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") };
        Node::Action(ActionKind::WaitUntil(cond, sel), vec![])
    }

    fn parse_login(&mut self) -> Node {
        let _line = self.pop().line;
        let u = match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") };
        let p = match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") };
        Node::Action(ActionKind::Login, vec![Expr::Str(u), Expr::Str(p)])
    }

    fn parse_stop(&mut self) -> Node { self.pop(); Node::Action(ActionKind::Stop, vec![]) }

    fn parse_write(&mut self) -> Node {
        let _line = self.pop().line;
        if matches!(self.peek().kind, TokenKind::File) {
            let obj = self.parse_object();
            let c = match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") };
            let p = match obj { ObjectRef::File(p) => p, _ => String::new() };
            Node::Action(ActionKind::Write, vec![Expr::Str(p), Expr::Str(c)])
        } else {
            let c = match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") };
            Node::Action(ActionKind::Write, vec![Expr::Str(String::new()), Expr::Str(c)])
        }
    }

    fn parse_transfer(&mut self) -> Node {
        let act = match self.pop().kind { TokenKind::Email => ActionKind::Email, _ => ActionKind::Upload };
        self.expect_ident("to");
        let t = match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") };
        let mut e = vec![Expr::Str(t)];
        if matches!(self.peek().kind, TokenKind::File) {
            if let ObjectRef::File(s) = self.parse_object() { e.push(Expr::Str(s)); }
        }
        Node::Action(act, e)
    }

    fn parse_ui_action(&mut self) -> Node {
        let k = self.pop().kind;
        if matches!(k, TokenKind::Click) {
            if matches!(self.peek().kind, TokenKind::String(_)) {
                Node::Action(ActionKind::Click, vec![Expr::Str(match self.pop().kind { TokenKind::String(s) => s, _ => unreachable!() })])
            } else { Node::Action(ActionKind::Click, vec![]) }
        } else {
            Node::Action(ActionKind::Find, vec![Expr::Str(match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") })])
        }
    }

    fn parse_find_all(&mut self) -> Node {
        let _line = self.pop().line; self.pop();
        Node::Action(ActionKind::FindAll, vec![Expr::Str(match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") })])
    }

    fn parse_get_number(&mut self) -> Node {
        let _line = self.pop().line; self.expect_ident("number");
        let s = if matches!(self.peek().kind, TokenKind::From) {
            self.pop(); Some(match self.pop().kind { TokenKind::String(s) => s, _ => "".into() })
        } else { None };
        Node::Action(ActionKind::GetNumber, vec![Expr::Str(s.unwrap_or_default())])
    }

    fn parse_log(&mut self) -> Node {
        let _line = self.pop().line; Node::ExprNode(self.parse_expr())
    }

    fn parse_when(&mut self) -> Node {
        let _line = self.pop().line; let c = self.parse_condition();
        self.skip_newlines(); self.expect_ident("do");
        let a = self.parse_actions(); self.expect_ident("done");
        Node::WhenNode(c, a, None)
    }

    fn parse_while(&mut self) -> Node {
        let _line = self.pop().line; let c = self.parse_condition();
        self.skip_newlines(); self.expect_ident("do");
        let a = self.parse_actions(); self.expect_ident("done");
        Node::WhileNode(c, a, None)
    }

    fn parse_condition(&mut self) -> Condition {
        let k = self.peek().kind.clone();
        if matches!(k, TokenKind::Item) {
            self.pop(); let c = token_name(&self.pop().kind);
            return if c == "visible" { Condition::ItemVisible } else { Condition::ItemHidden };
        }
        if matches!(k, TokenKind::NumKw) || matches!(k, TokenKind::Count) {
            let t = token_name(&self.pop().kind);
            let o = match &self.pop().kind { TokenKind::Op(s) => s.clone(), _ => panic!("expected operator") };
            let v = match self.pop().kind { TokenKind::Number(n) => n, _ => panic!("expected number") };
            return Condition::Compare(t, o, v);
        }
        Condition::Expression(self.parse_expr())
    }

    fn parse_let(&mut self) -> Node {
        let _line = self.pop().line;
        let n = match &self.pop().kind { TokenKind::Ident(s) => s.clone(), _ => panic!("expected identifier") };
        self.expect_ident("=");
        Node::LetStatement(n, self.parse_expr())
    }

    fn parse_fn(&mut self) -> Node {
        let _line = self.pop().line;
        let n = match &self.pop().kind { TokenKind::Ident(s) => s.clone(), _ => panic!("expected identifier") };
        let mut p = Vec::new();
        while matches!(self.peek().kind, TokenKind::Ident(_)) {
            p.push(match self.pop().kind { TokenKind::Ident(s) => s, _ => unreachable!() });
            self.skip_newlines();
        }
        self.expect_ident("do"); let a = self.parse_actions(); self.expect_ident("done");
        Node::FnDefinition(n, p, a)
    }

    fn parse_for(&mut self) -> Node {
        let _line = self.pop().line;
        let v = match &self.pop().kind { TokenKind::Ident(s) => s.clone(), _ => panic!("expected identifier") };
        self.expect_ident("in"); let col = self.parse_expr();
        self.skip_newlines(); self.expect_ident("do");
        let a = self.parse_actions(); self.expect_ident("done");
        Node::ForStatement(v, col, a, None)
    }

    fn parse_use(&mut self) -> Node {
        let _line = self.pop().line;
        Node::UseStatement(match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") })
    }

    fn parse_expr_stmt(&mut self) -> Node { Node::ExprNode(self.parse_expr()) }

    fn parse_expr(&mut self) -> Expr { self.parse_compare() }

    fn parse_compare(&mut self) -> Expr {
        let mut l = self.parse_addsub();
        loop {
            let k = self.peek().kind.clone();
            let op = match &k { TokenKind::Op(s) if s == ">" || s == "<" || s == ">=" || s == "<=" || s == "==" || s == "!=" || s == "=" => s.clone(), _ => break };
            self.pop();
            let r = self.parse_addsub();
            l = Expr::Bin(Box::new(l), match op.as_str() {
                ">" => Op::Gt, "<" => Op::Lt, ">=" => Op::Gte, "<=" => Op::Lte,
                "==" | "=" => Op::Eq, "!=" => Op::Neq, _ => unreachable!(),
            }, Box::new(r));
        }
        l
    }

    fn parse_addsub(&mut self) -> Expr {
        let mut l = self.parse_term();
        loop {
            let k = self.peek().kind.clone();
            let op = match &k { TokenKind::Op(s) if s == "+" || s == "-" => s.clone(), _ => break };
            self.pop(); let r = self.parse_term();
            l = Expr::Bin(Box::new(l), if op == "+" { Op::Add } else { Op::Sub }, Box::new(r));
        }
        l
    }

    fn parse_term(&mut self) -> Expr {
        let mut l = self.parse_unary();
        loop {
            let k = self.peek().kind.clone();
            let op = match &k { TokenKind::Op(s) if s == "*" || s == "/" => s.clone(), _ => break };
            self.pop(); let r = self.parse_unary();
            l = Expr::Bin(Box::new(l), if op == "*" { Op::Mul } else { Op::Div }, Box::new(r));
        }
        l
    }

    fn parse_unary(&mut self) -> Expr {
        if matches!(self.peek().kind, TokenKind::Op(ref s) if s == "-") {
            self.pop(); return Expr::Bin(Box::new(Expr::Num(-1)), Op::Mul, Box::new(self.parse_unary()));
        }
        if matches!(self.peek().kind, TokenKind::Not) {
            self.pop(); return Expr::Not(Box::new(self.parse_unary()));
        }
        self.parse_factor()
    }

    fn parse_factor(&mut self) -> Expr {
        let k = self.peek().kind.clone();
        let result: Expr = match k {
            TokenKind::Number(n) => { self.pop(); Expr::Num(n) }
            TokenKind::String(s) => { self.pop(); Expr::Str(s) }
            TokenKind::Run => {
                self.pop(); let cmd = match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") };
                let stdin = if matches!(self.peek().kind, TokenKind::With) { self.pop(); Some(Box::new(self.parse_expr())) } else { None };
                Expr::Run(cmd, stdin)
            }
            TokenKind::Read => { self.pop(); Expr::Read(match self.pop().kind { TokenKind::String(s) => s, _ => panic!("expected string") }) }
            TokenKind::Ls => {
                self.pop();
                let p = if matches!(self.peek().kind, TokenKind::String(_)) { match self.pop().kind { TokenKind::String(s) => s, _ => unreachable!() } } else { "*".into() };
                Expr::Ls(p)
            }
            TokenKind::Len => { self.pop(); Expr::Len(Box::new(self.parse_expr())) }
            TokenKind::LBracket => {
                self.pop();
                let mut items = Vec::new();
                if !matches!(self.peek().kind, TokenKind::RBracket) {
                    items.push(self.parse_expr());
                    while matches!(self.peek().kind, TokenKind::Comma) { self.pop(); items.push(self.parse_expr()); }
                }
                self.expect_ident("]");
                Expr::List(items)
            }
            TokenKind::Op(ref s) if s == "(" => { self.pop(); let e = self.parse_expr(); self.expect_ident(")"); e }
            TokenKind::Ident(s) => {
                self.pop();
                if matches!(self.peek().kind, TokenKind::LBracket) { Expr::Var(s) }
                else if is_expr_start(&self.peek().kind) { Expr::Call(s, vec![self.parse_expr()]) }
                else { Expr::Var(s) }
            }
            TokenKind::Item | TokenKind::Count | TokenKind::NumKw => {
                let name = token_name(&k); self.pop();
                Expr::Var(name)
            }
            _ => panic!("line {}: unexpected token {:?}", self.peek().line, k),
        };

        let mut cur = result;
        loop {
            if matches!(self.peek().kind, TokenKind::LBracket) {
                self.pop(); let s = self.parse_expr();
                if matches!(self.peek().kind, TokenKind::DotDot) {
                    self.pop(); let e = self.parse_expr(); self.expect_ident("]");
                    cur = Expr::Slice(Box::new(cur), Box::new(s), Box::new(e));
                } else { self.expect_ident("]"); cur = Expr::Index(Box::new(cur), Box::new(s)); }
            } else if matches!(self.peek().kind, TokenKind::Op(ref o) if o == ".") {
                self.pop();
                let m = match &self.pop().kind { TokenKind::Ident(s) => s.clone(), _ => panic!("expected method name") };
                let mut a = Vec::new();
                if is_expr_start(&self.peek().kind) { a.push(self.parse_expr()); }
                cur = Expr::Method(Box::new(cur), m, a);
            } else { break; }
        }
        cur
    }
}

fn set_fallback(node: &mut Node, fb: Vec<Node>) {
    match node {
        Node::TimeNode(_, _, ref mut f) | Node::ScriptNode(_, ref mut f)
        | Node::WithNode(_, _, _, ref mut f) | Node::RetryNode(_, _, ref mut f)
        | Node::WhenNode(_, _, ref mut f) | Node::WhileNode(_, _, ref mut f)
        | Node::WatchNode(_, _, ref mut f) | Node::ForStatement(_, _, _, ref mut f) => {
            *f = Some(fb);
        }
        _ => {}
    }
}
