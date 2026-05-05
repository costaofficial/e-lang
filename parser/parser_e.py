"""
E parser — general-purpose language
=====================================
Lexer + Recursive Descent Parser → AST

Usage:
    python3 parser/parser_e.py script.e
"""

import sys
import re
from dataclasses import dataclass, field
from typing import Optional


# ──────────────────────────────────────────────
# AST Nodes
# ──────────────────────────────────────────────

@dataclass
class Program:
    blocks: list

@dataclass
class TimeBlock:
    schedule: 'Schedule'
    actions: list
    fallback: Optional = None
    line: int = 0

@dataclass
class ScriptBlock:
    actions: list
    fallback: Optional = None
    line: int = 0

@dataclass
class Schedule:
    kind: str
    interval: Optional = None
    time: Optional = None
    line: int = 0

@dataclass
class WithBlock:
    object: 'ObjectRef'
    config: Optional = None
    actions: list = field(default_factory=list)
    fallback: Optional = None
    line: int = 0

@dataclass
class ObjectRef:
    kind: str
    value: Optional[str] = None
    line: int = 0

@dataclass
class RetryBlock:
    times: int
    actions: list
    fallback: Optional = None
    line: int = 0

@dataclass
class WatchBlock:
    path: str
    actions: list
    fallback: Optional = None
    line: int = 0

@dataclass
class Action:
    kind: str
    args: list = field(default_factory=list)
    fallback: Optional = None
    line: int = 0


# ──────────────────────────────────────────────
# Lexer
# ──────────────────────────────────────────────

KEYWORDS = {
    'time', 'every', 'at', 'do', 'done', 'with', 'or',
    'retry', 'times', 'wait', 'until', 'watch',
    'login', 'stop', 'write', 'email', 'upload',
    'click', 'find', 'log',
    'file', 'browser', 'page', 'app',
    'visible', 'hidden', 'download', 'to',
    's', 'ms', 'timeout',
}

TOKEN_SPEC = [
    ('COMMENT',   r'//[^\n]*'),
    ('STRING',    r'"[^"]*"'),
    ('NUMBER',    r'\d+'),
    ('IDENT',     r'[a-zA-Z_][a-zA-Z0-9_]*'),
    ('LBRACE',    r'\{'),
    ('RBRACE',    r'\}'),
    ('COLON',     r':'),
    ('NEWLINE',   r'\n+'),
    ('SKIP',      r'[ \t]+'),
    ('MISMATCH',  r'.'),
]

TOKEN_RE = re.compile('|'.join(f'(?P<{name}>{pattern})' for name, pattern in TOKEN_SPEC))


@dataclass
class Token:
    kind: str
    value: str
    line: int


def lex(source: str):
    tokens = []
    line = 1
    for m in TOKEN_RE.finditer(source):
        kind = m.lastgroup
        val = m.group()
        if kind == 'COMMENT':
            pass
        elif kind == 'SKIP':
            pass
        elif kind == 'NEWLINE':
            line += val.count('\n')
        elif kind == 'IDENT':
            tag = 'KEYWORD' if val in KEYWORDS else 'IDENT'
            tokens.append(Token(tag, val, line))
        elif kind == 'MISMATCH':
            raise SyntaxError(f"unknown character '{val}' at line {line}")
        else:
            tokens.append(Token(kind, val, line))
    return tokens


# ──────────────────────────────────────────────
# Parser
# ──────────────────────────────────────────────

class ParseError(SyntaxError):
    pass


class Parser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.pos = 0

    def peek(self):
        return self.tokens[self.pos] if self.pos < len(self.tokens) else Token('EOF', '', 0)

    def pop(self):
        t = self.peek()
        self.pos += 1
        return t

    def expect(self, kind, value=None):
        t = self.pop()
        if t.kind != kind or (value is not None and t.value != value):
            expected = repr(value) if value else kind
            raise ParseError(f"line {t.line}: expected {expected}, got '{t.value}'")
        return t

    def maybe(self, kind, value=None):
        t = self.peek()
        if t.kind == kind and (value is None or t.value == value):
            return self.pop()
        return None

    def skip_newlines(self):
        while self.peek().kind == 'NEWLINE':
            self.pop()

    # ──── Parsing methods ────

    def parse(self):
        blocks = []
        self.skip_newlines()
        while self.peek().kind != 'EOF':
            blk = self.parse_statement_block()
            if blk:
                blocks.append(blk)
            self.skip_newlines()
        return Program(blocks)

    def parse_statement_block(self):
        t = self.peek()
        if t.value == 'time':
            return self.parse_time_block()
        elif t.value == 'do':
            return self.parse_script_block()
        raise ParseError(f"line {t.line}: expected 'time' or 'do', got '{t.value}'")

    def parse_time_block(self):
        line = self.pop().line
        sched = self.parse_schedule()
        self.expect('KEYWORD', 'do')
        actions = self.parse_actions()
        self.expect('KEYWORD', 'done')
        fallback = self.parse_optional_fallback()
        return TimeBlock(sched, actions, fallback, line=line)

    def parse_schedule(self):
        t = self.peek()
        line = t.line
        if t.value == 'every':
            self.pop()
            interval = self.pop().value
            if self.peek().value == 'at':
                self.pop()
                time_val = self.parse_time()
            else:
                time_val = None
            return Schedule('every', interval, time_val, line=line)
        elif t.value == 'at':
            self.pop()
            return Schedule('at', time=self.parse_time(), line=line)
        raise ParseError(f"line {t.line}: expected 'every' or 'at', got '{t.value}'")

    def parse_time(self):
        h = self.expect('NUMBER').value
        if self.peek().kind == 'COLON':
            self.pop()
            m = self.expect('NUMBER').value
            return f"{h}:{m}"
        return f"{h}:00"

    def parse_script_block(self):
        line = self.pop().line
        actions = self.parse_actions()
        self.expect('KEYWORD', 'done')
        fallback = self.parse_optional_fallback()
        return ScriptBlock(actions, fallback, line=line)

    def parse_actions(self):
        actions = []
        self.skip_newlines()
        while self.peek().value not in ('done', 'EOF') and self.peek().kind != 'EOF':
            if self.peek().value == 'or':
                break
            stmt = self.parse_statement()
            if stmt:
                actions.append(stmt)
            self.skip_newlines()
        return actions

    def parse_statement(self):
        core = self.parse_core_statement()
        self.skip_newlines()
        if self.peek().value == 'or':
            core.fallback = self.parse_fallback()
        return core

    SIMPLE_ACTIONS = {'open', 'run', 'create'}

    def parse_core_statement(self):
        t = self.peek()
        if t.value == 'with':
            return self.parse_with_block()
        elif t.value == 'retry':
            return self.parse_retry_block()
        elif t.value == 'watch':
            return self.parse_watch_block()
        elif t.value == 'wait':
            return self.parse_wait_statement()
        elif t.value == 'login':
            return self.parse_login_statement()
        elif t.value == 'stop':
            return self.parse_stop_statement()
        elif t.value == 'write':
            return self.parse_write_action()
        elif t.value in ('email', 'upload'):
            return self.parse_transfer_action()
        elif t.value in ('click', 'find'):
            return self.parse_ui_action()
        elif t.value == 'log':
            return self.parse_log_action()
        elif t.value in self.SIMPLE_ACTIONS:
            return self.parse_simple_action()
        raise ParseError(f"line {t.line}: unknown action '{t.value}'")

    def parse_simple_action(self):
        t = self.pop()
        arg = self.expect('STRING').value.strip('"')
        return Action(t.value, [arg], line=t.line)

    def parse_with_block(self):
        line = self.pop().line
        obj = self.parse_object()
        config = None
        if self.peek().kind == 'LBRACE':
            self.pop()
            self.expect('KEYWORD', 'timeout')
            self.expect('COLON', ':')
            dur = self.pop()
            config = dur.value
            if self.peek().kind == 'KEYWORD' and self.peek().value in ('s', 'ms'):
                config += self.pop().value
            self.expect('RBRACE', '}')
        self.expect('KEYWORD', 'do')
        actions = self.parse_actions()
        self.expect('KEYWORD', 'done')
        return WithBlock(obj, config, actions, line=line)

    def parse_object(self):
        t = self.pop()
        if t.value in ('file', 'app'):
            name = self.expect('STRING').value.strip('"')
            return ObjectRef(t.value, name, line=t.line)
        elif t.value in ('browser', 'page'):
            return ObjectRef(t.value, line=t.line)
        raise ParseError(f"line {t.line}: unknown object '{t.value}'")

    def parse_retry_block(self):
        line = self.pop().line
        n = int(self.expect('NUMBER').value)
        self.expect('KEYWORD', 'times')
        self.expect('KEYWORD', 'do')
        actions = self.parse_actions()
        self.expect('KEYWORD', 'done')
        return RetryBlock(n, actions, line=line)

    def parse_watch_block(self):
        line = self.pop().line
        path = self.expect('STRING').value.strip('"')
        self.expect('KEYWORD', 'do')
        actions = self.parse_actions()
        self.expect('KEYWORD', 'done')
        return WatchBlock(path, actions, line=line)

    def parse_wait_statement(self):
        line = self.pop().line
        if self.peek().value == 'download':
            self.pop()
            return Action('wait_download', line=line)
        self.expect('KEYWORD', 'until')
        t = self.peek()
        if t.value in ('visible', 'hidden'):
            cond = self.pop().value
            sel = self.expect('STRING').value.strip('"')
            return Action('wait_until', [cond, sel], line=line)
        raise ParseError(f"line {t.line}: expected 'visible', 'hidden' or 'download'")

    def parse_login_statement(self):
        line = self.pop().line
        user = self.expect('STRING').value.strip('"')
        pwd = self.expect('STRING').value.strip('"')
        return Action('login', [user, pwd], line=line)

    def parse_stop_statement(self):
        line = self.pop().line
        return Action('stop', line=line)

    def parse_write_action(self):
        line = self.pop().line
        if self.peek().value == 'file':
            obj = self.parse_object()
            content = self.expect('STRING').value.strip('"')
            return Action('write', [obj, content], line=line)
        content = self.expect('STRING').value.strip('"')
        return Action('write', [None, content], line=line)

    def parse_transfer_action(self):
        t = self.pop()
        line, kind = t.line, t.value
        self.expect('KEYWORD', 'to')
        target = self.expect('STRING').value.strip('"')
        obj = None
        if self.peek().value == 'file':
            obj = self.parse_object()
        return Action(kind, [target, obj], line=line)

    def parse_ui_action(self):
        t = self.pop()
        line, kind = t.line, t.value
        if kind == 'find':
            sel = self.expect('STRING').value.strip('"')
            return Action('find', [sel], line=line)
        if self.peek().kind == 'STRING':
            sel = self.pop().value.strip('"')
            return Action('click', [sel], line=line)
        return Action('click', line=line)

    def parse_log_action(self):
        line = self.pop().line
        t = self.peek()
        if t.kind == 'STRING':
            msg = self.pop().value.strip('"')
        elif t.kind in ('KEYWORD', 'IDENT'):
            msg = self.pop().value
        else:
            raise ParseError(f"line {t.line}: expected string or identifier for log")
        return Action('log', [msg], line=line)

    def parse_optional_fallback(self):
        self.skip_newlines()
        if self.peek().value != 'or':
            return None
        return self.parse_fallback()

    def parse_fallback(self):
        self.pop()  # 'or'
        self.skip_newlines()
        if self.peek().value == 'do':
            self.pop()
            actions = self.parse_actions()
            self.expect('KEYWORD', 'done')
            return actions
        return [self.parse_core_statement()]


# ──────────────────────────────────────────────
# Pretty Printer (AST dump)
# ──────────────────────────────────────────────

def dump(node, indent=0):
    pad = "  " * indent
    if isinstance(node, Program):
        print(f"{pad}Program")
        for b in node.blocks:
            dump(b, indent + 1)
    elif isinstance(node, TimeBlock):
        print(f"{pad}TimeBlock [line {node.line}]")
        dump(node.schedule, indent + 1)
        print(f"{pad}  Actions:")
        for a in node.actions:
            dump(a, indent + 2)
        if node.fallback:
            print(f"{pad}  Fallback:")
            for f in node.fallback:
                dump(f, indent + 2)
    elif isinstance(node, ScriptBlock):
        print(f"{pad}ScriptBlock [line {node.line}]")
        print(f"{pad}  Actions:")
        for a in node.actions:
            dump(a, indent + 2)
        if node.fallback:
            print(f"{pad}  Fallback:")
            for f in node.fallback:
                dump(f, indent + 2)
    elif isinstance(node, Schedule):
        print(f"{pad}Schedule({node.kind}, interval={node.interval}, time={node.time})")
    elif isinstance(node, WithBlock):
        print(f"{pad}WithBlock [line {node.line}] config={node.config}")
        dump(node.object, indent + 1)
        print(f"{pad}  Actions:")
        for a in node.actions:
            dump(a, indent + 2)
    elif isinstance(node, ObjectRef):
        print(f"{pad}Object({node.kind}, {node.value})")
    elif isinstance(node, RetryBlock):
        print(f"{pad}RetryBlock({node.times}x) [line {node.line}]")
        for a in node.actions:
            dump(a, indent + 1)
        if node.fallback:
            print(f"{pad}  Fallback:")
            for f in node.fallback:
                dump(f, indent + 2)
    elif isinstance(node, WatchBlock):
        print(f"{pad}WatchBlock(path={node.path}) [line {node.line}]")
        for a in node.actions:
            dump(a, indent + 1)
    elif isinstance(node, Action):
        args_str = ", ".join(repr(a) if not isinstance(a, str) else a for a in node.args)
        fb = " [or fallback]" if node.fallback else ""
        print(f"{pad}{node.kind}({args_str}){fb} [line {node.line}]")
        if node.fallback:
            for f in node.fallback:
                dump(f, indent + 1)
    else:
        print(f"{pad}{node!r}")


# ──────────────────────────────────────────────
# Main
# ──────────────────────────────────────────────

def main():
    if len(sys.argv) < 2:
        print("Usage: python parser_e.py <file.e>")
        sys.exit(1)

    for path in sys.argv[1:]:
        print(f"\n{'='*60}")
        print(f"📄 {path}")
        print(f"{'='*60}")
        try:
            with open(path) as f:
                source = f.read()
            tokens = lex(source)
            parser = Parser(tokens)
            ast = parser.parse()
            dump(ast)
        except (SyntaxError, ParseError) as e:
            print(f"❌ ERROR: {e}")
        except FileNotFoundError:
            print(f"❌ File not found: {path}")


if __name__ == '__main__':
    main()
