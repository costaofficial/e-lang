use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Time, Every, At, Do, Done, With, Or,
    Retry, Times, Wait, Until, Watch,
    Login, Stop, Write, Email, Upload,
    Click, Find, Log,
    File, Browser, Page, App,
    Visible, Hidden, Download, To,
    Timeout,
    When, All, Get, From, NumKw, Item, Count,
    Let, Fn, Run, Read, Ls,
    For, In, Use, Append,
    While, Len, Not,
    Hour, Day, Minute, S, Ms,

    Number(i64),
    String(String),
    Ident(String),

    Op(String),
    LBrace, RBrace,
    LBracket, RBracket,
    Comma, Colon,
    DotDot,
    Newline,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
}

fn key(s: &str) -> Option<TokenKind> {
    Some(match s {
        "time" => TokenKind::Time, "every" => TokenKind::Every,
        "at" => TokenKind::At, "do" => TokenKind::Do,
        "done" => TokenKind::Done, "with" => TokenKind::With,
        "or" => TokenKind::Or, "retry" => TokenKind::Retry,
        "times" => TokenKind::Times, "wait" => TokenKind::Wait,
        "until" => TokenKind::Until, "watch" => TokenKind::Watch,
        "login" => TokenKind::Login, "stop" => TokenKind::Stop,
        "write" => TokenKind::Write, "email" => TokenKind::Email,
        "upload" => TokenKind::Upload, "click" => TokenKind::Click,
        "find" => TokenKind::Find, "log" => TokenKind::Log,
        "file" => TokenKind::File, "browser" => TokenKind::Browser,
        "page" => TokenKind::Page, "app" => TokenKind::App,
        "visible" => TokenKind::Visible, "hidden" => TokenKind::Hidden,
        "download" => TokenKind::Download, "to" => TokenKind::To,
        "timeout" => TokenKind::Timeout, "when" => TokenKind::When,
        "all" => TokenKind::All, "get" => TokenKind::Get,
        "from" => TokenKind::From, "number" => TokenKind::NumKw,
        "item" => TokenKind::Item, "count" => TokenKind::Count,
        "let" => TokenKind::Let, "fn" => TokenKind::Fn,
        "run" => TokenKind::Run, "read" => TokenKind::Read,
        "ls" => TokenKind::Ls, "for" => TokenKind::For,
        "in" => TokenKind::In, "use" => TokenKind::Use,
        "append" => TokenKind::Append, "while" => TokenKind::While,
        "len" => TokenKind::Len, "not" => TokenKind::Not,
        "hour" => TokenKind::Hour, "day" => TokenKind::Day,
        "minute" => TokenKind::Minute, "s" => TokenKind::S,
        "ms" => TokenKind::Ms,
        _ => return None,
    })
}

pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let c: Vec<char> = source.chars().collect();
    let mut i = 0;
    let mut line = 1;

    while i < c.len() {
        match c[i] {
            ' ' | '\t' => { i += 1; }
            '\n' => { line += 1; i += 1; tokens.push(Token { kind: TokenKind::Newline, line }); }
            '/' if i + 1 < c.len() && c[i+1] == '/' => {
                while i < c.len() && c[i] != '\n' { i += 1; }
            }
            '"' => {
                i += 1; let s = i;
                while i < c.len() && c[i] != '"' { if c[i] == '\n' { line += 1; } i += 1; }
                if i >= c.len() { return Err(format!("unterminated string at line {}", line)); }
                let val: String = c[s..i].iter().collect(); i += 1;
                tokens.push(Token { kind: TokenKind::String(val), line });
            }
            ch if ch.is_ascii_digit() => {
                let s = i;
                while i < c.len() && c[i].is_ascii_digit() { i += 1; }
                let n: String = c[s..i].iter().collect();
                tokens.push(Token { kind: TokenKind::Number(n.parse().unwrap()), line });
            }
            ch if ch.is_ascii_alphabetic() || ch == '_' => {
                let s = i;
                while i < c.len() && (c[i].is_ascii_alphanumeric() || c[i] == '_') { i += 1; }
                let w: String = c[s..i].iter().collect();
                tokens.push(Token { kind: key(&w).unwrap_or(TokenKind::Ident(w)), line });
            }
            '{' => { i += 1; tokens.push(Token { kind: TokenKind::LBrace, line }); }
            '}' => { i += 1; tokens.push(Token { kind: TokenKind::RBrace, line }); }
            '[' => { i += 1; tokens.push(Token { kind: TokenKind::LBracket, line }); }
            ']' => { i += 1; tokens.push(Token { kind: TokenKind::RBracket, line }); }
            ',' => { i += 1; tokens.push(Token { kind: TokenKind::Comma, line }); }
            ':' => { i += 1; tokens.push(Token { kind: TokenKind::Colon, line }); }
            '(' => { i += 1; tokens.push(Token { kind: TokenKind::Op("(".into()), line }); }
            ')' => { i += 1; tokens.push(Token { kind: TokenKind::Op(")".into()), line }); }
            '+' => { i += 1; tokens.push(Token { kind: TokenKind::Op("+".into()), line }); }
            '-' => { i += 1; tokens.push(Token { kind: TokenKind::Op("-".into()), line }); }
            '*' => { i += 1; tokens.push(Token { kind: TokenKind::Op("*".into()), line }); }
            '/' => { i += 1; tokens.push(Token { kind: TokenKind::Op("/".into()), line }); }
            '.' if i+1 < c.len() && c[i+1] == '.' => {
                i += 2; tokens.push(Token { kind: TokenKind::DotDot, line });
            }
            '.' => { i += 1; tokens.push(Token { kind: TokenKind::Op(".".into()), line }); }
            '>' if i+1 < c.len() && c[i+1] == '=' => { i += 2; tokens.push(Token { kind: TokenKind::Op(">=".into()), line }); }
            '<' if i+1 < c.len() && c[i+1] == '=' => { i += 2; tokens.push(Token { kind: TokenKind::Op("<=".into()), line }); }
            '>' => { i += 1; tokens.push(Token { kind: TokenKind::Op(">".into()), line }); }
            '<' => { i += 1; tokens.push(Token { kind: TokenKind::Op("<".into()), line }); }
            '=' if i+1 < c.len() && c[i+1] == '=' => { i += 2; tokens.push(Token { kind: TokenKind::Op("==".into()), line }); }
            '!' if i+1 < c.len() && c[i+1] == '=' => { i += 2; tokens.push(Token { kind: TokenKind::Op("!=".into()), line }); }
            '=' => { i += 1; tokens.push(Token { kind: TokenKind::Op("=".into()), line }); }
            ch => return Err(format!("unknown character '{}' at line {}", ch, line)),
        }
    }

    tokens.push(Token { kind: TokenKind::Eof, line });
    Ok(tokens)
}
