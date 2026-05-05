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
class TimeUnit:
    schedule: 'Schedule'
    actions: list
    fallback: Optional = None
    line: int = 0

@dataclass
class ScriptUnit:
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
class WithUnit:
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
class RetryUnit:
    times: int
    actions: list
    fallback: Optional = None
    line: int = 0

@dataclass
class WatchUnit:
    path: str
    actions: list
    fallback: Optional = None
    line: int = 0

@dataclass
class WhenUnit:
    condition: dict
    actions: list
    fallback: Optional = None
    line: int = 0

@dataclass
class Action:
    kind: str
    args: list = field(default_factory=list)
    fallback: Optional = None
    line: int = 0

@dataclass
class LetStatement:
    name: str
    value: 'Expr'
    line: int = 0

@dataclass
class FnDefinition:
    name: str
    params: list
    body: list
    line: int = 0

@dataclass
class ForStatement:
    var: str
    collection: 'Expr'
    body: list
    fallback: Optional = None
    line: int = 0

@dataclass
class ImportStatement:
    path: str
    line: int = 0

@dataclass
class Expr:
    kind: str   # 'num' | 'str' | 'var' | 'call' | 'bin'
    value: any = None
    left: Optional['Expr'] = None
    right: Optional['Expr'] = None
    op: Optional[str] = None
    args: list = field(default_factory=list)
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
    'when', 'all', 'get', 'from', 'number', 'item', 'count',
    'let', 'fn', 'run', 'read', 'ls',
    'for', 'in', 'import', 'append',
}

TOKEN_SPEC = [
    ('COMMENT',   r'//[^\n]*'),
    ('STRING',    r'"[^"]*"'),
    ('NUMBER',    r'\d+'),
    ('IDENT',     r'[a-zA-Z_][a-zA-Z0-9_]*'),
    ('OP',        r'>=|<=|>|<|==|!=|=|\+|\-|\*|/|\.'),
    ('LBRACKET',  r'\['),
    ('RBRACKET',  r'\]'),
    ('COMMA',     r','),
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
        elif t.value == 'fn':
            return self.parse_fn()
        elif t.value == 'let':
            return self.parse_let()
        elif t.value == 'import':
            return self.parse_import()
        raise ParseError(f"line {t.line}: expected 'time' or 'do', got '{t.value}'")

    def parse_time_block(self):
        line = self.pop().line
        sched = self.parse_schedule()
        self.expect('KEYWORD', 'do')
        actions = self.parse_actions()
        self.expect('KEYWORD', 'done')
        fallback = self.parse_optional_fallback()
        return TimeUnit(sched, actions, fallback, line=line)

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
        return ScriptUnit(actions, fallback, line=line)

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
        if self.peek().value == 'or' and hasattr(core, 'fallback'):
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
        elif t.value == 'find' and self._peek_next().value == 'all':
            return self.parse_find_all()
        elif t.value in ('click', 'find'):
            return self.parse_ui_action()
        elif t.value == 'log':
            return self.parse_log_action()
        elif t.value in self.SIMPLE_ACTIONS:
            return self.parse_simple_action()
        elif t.value == 'when':
            return self.parse_when_block()
        elif t.value == 'get' and self._peek_next().value == 'number':
            return self.parse_get_number()
        elif t.value == 'let':
            return self.parse_let()
        elif t.value == 'fn':
            return self.parse_fn()
        elif t.value == 'for':
            return self.parse_for()
        elif t.value == 'import':
            return self.parse_import()
        elif t.kind in ('IDENT', 'NUMBER', 'STRING', 'LBRACKET') or \
             (t.kind == 'KEYWORD' and t.value in ('number', 'count', 'item', 'run', 'read', 'ls')):
            return self.parse_expr_stmt()
        raise ParseError(f"line {t.line}: unknown action '{t.value}'")

    def _peek_next(self):
        """Peek at the next real token (skip newlines)."""
        pos = self.pos + 1
        while pos < len(self.tokens) and self.tokens[pos].kind == 'NEWLINE':
            pos += 1
        return self.tokens[pos] if pos < len(self.tokens) else Token('EOF', '', 0)

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
        return WithUnit(obj, config, actions, line=line)

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
        return RetryUnit(n, actions, line=line)

    def parse_watch_block(self):
        line = self.pop().line
        path = self.expect('STRING').value.strip('"')
        self.expect('KEYWORD', 'do')
        actions = self.parse_actions()
        self.expect('KEYWORD', 'done')
        return WatchUnit(path, actions, line=line)

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

    def parse_when_block(self):
        line = self.pop().line  # 'when'
        cond = self.parse_condition()
        self.expect('KEYWORD', 'do')
        actions = self.parse_actions()
        self.expect('KEYWORD', 'done')
        return WhenUnit(cond, actions, line=line)

    def parse_condition(self):
        t = self.peek()
        # Special semantic variables: item, number, count
        if t.value in ('item', 'number', 'count'):
            self.pop()
            target = t.value
            if target == 'item':
                cond = self.pop().value  # 'visible' or 'hidden'
                return {'type': f'item_{cond}', 'target': 'item'}
            elif target in ('number', 'count'):
                op = self.expect('OP').value
                val = int(self.expect('NUMBER').value)
                return {'type': 'compare', 'target': target, 'operator': op, 'value': val}
        # General expression
        expr = self.parse_expr()
        return {'type': 'expr', 'expr': expr}

    def parse_get_number(self):
        line = self.pop().line  # 'get'
        self.expect('KEYWORD', 'number')
        selector = None
        if self.peek().value == 'from':
            self.pop()
            selector = self.expect('STRING').value.strip('"')
        return Action('get_number', [selector], line=line)

    def parse_find_all(self):
        t = self.pop()  # 'find'
        self.pop()  # 'all'
        selector = self.expect('STRING').value.strip('"')
        return Action('find_all', [selector], line=t.line)

    # ── Variables & Functions ──

    def parse_let(self):
        line = self.pop().line  # 'let'
        name = self.expect('IDENT').value
        self.expect('OP', '=')
        val = self.parse_expr()
        return LetStatement(name, val, line=line)

    def parse_fn(self):
        line = self.pop().line  # 'fn'
        name = self.expect('IDENT').value
        params = []
        while self.peek().kind == 'IDENT':
            params.append(self.pop().value)
            self.skip_newlines()
        self.expect('KEYWORD', 'do')
        actions = self.parse_actions()
        self.expect('KEYWORD', 'done')
        return FnDefinition(name, params, actions, line=line)

    def parse_expr_stmt(self):
        """Parse an expression used as a statement (bare expr or fn call)."""
        line = self.peek().line
        expr = self.parse_expr()
        if expr.kind == 'call' or expr.kind == 'var' or expr.kind == 'bin':
            return expr
        return expr

    # ── Expressions ──

    def parse_expr(self):
        return self.parse_compare()

    def parse_compare(self):
        left = self.parse_addsub()
        while self.peek().kind == 'OP' and self.peek().value in ('>', '<', '>=', '<=', '==', '!='):
            op = self.pop().value
            right = self.parse_addsub()
            left = Expr('bin', op=op, left=left, right=right, line=left.line)
        return left

    def parse_addsub(self):
        left = self.parse_term()
        while self.peek().kind == 'OP' and self.peek().value in ('+', '-', 'or', 'and'):
            op = self.pop().value
            right = self.parse_term()
            left = Expr('bin', op=op, left=left, right=right, line=left.line)
        return left

    def parse_term(self):
        left = self.parse_unary()
        while self.peek().kind == 'OP' and self.peek().value in ('*', '/'):
            op = self.pop().value
            right = self.parse_unary()
            left = Expr('bin', op=op, left=left, right=right, line=left.line)
        return left

    def parse_unary(self):
        if self.peek().kind == 'OP' and self.peek().value == '-':
            line = self.pop().line
            right = self.parse_unary()
            return Expr('bin', op='*', left=Expr('num', -1, line=line), right=right, line=line)
        return self.parse_factor()

    def parse_factor(self):
        t = self.peek()
        line = t.line
        result = None

        if t.kind == 'NUMBER':
            self.pop()
            result = Expr('num', int(t.value), line=line)

        elif t.kind == 'STRING':
            self.pop()
            result = Expr('str', t.value.strip('"'), line=line)

        elif t.value == 'run':
            self.pop()
            cmd = self.expect('STRING').value.strip('"')
            stdin_expr = None
            if self.peek().value == 'with':
                self.pop()
                stdin_expr = self.parse_expr()
            result = Expr('run', cmd, args=[stdin_expr] if stdin_expr else [], line=line)

        elif t.kind == 'LBRACKET':
            self.pop()
            items = []
            if self.peek().kind != 'RBRACKET':
                items.append(self.parse_expr())
                while self.peek().kind == 'COMMA':
                    self.pop()
                    items.append(self.parse_expr())
            self.expect('RBRACKET', ']')
            result = Expr('list', items, line=line)

        elif t.value == 'read':
            self.pop()
            path = self.expect('STRING').value.strip('"')
            result = Expr('read', path, line=line)

        elif t.value == 'ls':
            self.pop()
            pattern = '*'
            if self.peek().kind == 'STRING':
                pattern = self.pop().value.strip('"')
            result = Expr('ls', pattern, line=line)

        elif t.kind == 'IDENT' or (t.kind == 'KEYWORD' and t.value in ('number', 'count', 'item')):
            self.pop()
            name = t.value
            # If next is '[', this is indexing, not a function call
            if self.peek().kind == 'LBRACKET':
                result = Expr('var', name, line=line)
            elif self.peek().kind in ('NUMBER', 'STRING', 'IDENT', 'LBRACKET') or \
               (self.peek().kind == 'KEYWORD' and self.peek().value in ('number', 'count', 'item')) or \
               (self.peek().kind == 'OP' and self.peek().value == '-'):
                args = [self.parse_expr()]
                result = Expr('call', name, args=args, line=line)
            else:
                result = Expr('var', name, line=line)

        elif t.kind == 'OP' and t.value == '(':
            self.pop()
            result = self.parse_expr()
            self.expect('OP', ')')

        else:
            raise ParseError(f"line {t.line}: unexpected token '{t.value}' in expression")

        # Postfix: indexing [n] and method calls .method
        while result is not None:
            if self.peek().kind == 'LBRACKET':
                self.pop()
                index = self.parse_expr()
                self.expect('RBRACKET', ']')
                result = Expr('index', left=result, right=index, line=line)
            elif self.peek().kind == 'OP' and self.peek().value == '.':
                self.pop()
                t = self.pop()
                if t.kind not in ('IDENT', 'KEYWORD'):
                    raise ParseError(f"line {t.line}: expected method name, got '{t.value}'")
                method = t.value
                args = []
                if self.peek().kind == 'LBRACKET':
                    self.pop()
                    if self.peek().kind != 'RBRACKET':
                        args.append(self.parse_expr())
                        while self.peek().kind == 'COMMA':
                            self.pop()
                            args.append(self.parse_expr())
                    self.expect('RBRACKET', ']')
                elif self.peek().kind in ('NUMBER', 'STRING', 'IDENT', 'LBRACKET') or \
                     (self.peek().kind == 'KEYWORD' and self.peek().value in ('number', 'count', 'item', 'run', 'read', 'ls')):
                    args.append(self.parse_expr())
                result = Expr('method', value=method, left=result, args=args, line=line)
            else:
                break

        return result

    def parse_for(self):
        line = self.pop().line  # 'for'
        t = self.pop()
        if t.kind not in ('IDENT', 'KEYWORD'):
            raise ParseError(f"line {t.line}: expected variable name, got '{t.value}'")
        var = t.value
        self.expect('KEYWORD', 'in')
        collection = self.parse_expr()
        self.skip_newlines()
        self.expect('KEYWORD', 'do')
        actions = self.parse_actions()
        self.expect('KEYWORD', 'done')
        return ForStatement(var, collection, actions, line=line)

    def parse_import(self):
        line = self.pop().line  # 'import'
        path = self.expect('STRING').value.strip('"')
        return ImportStatement(path, line=line)

    def parse_log_action(self):
        line = self.pop().line
        expr = self.parse_expr()
        if expr.kind == 'str':
            return Action('log', [expr.value], line=line)
        return Action('log_expr', [expr], line=line)

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

def dump_expr(e: Expr) -> str:
    if e.kind == 'num':
        return str(e.value)
    elif e.kind == 'str':
        return f'"{e.value}"'
    elif e.kind == 'var':
        return e.value
    elif e.kind == 'call':
        return f"{e.value}({', '.join(dump_expr(a) for a in e.args)})"
    elif e.kind == 'bin':
        return f"({dump_expr(e.left)} {e.op} {dump_expr(e.right)})"
    elif e.kind == 'run':
        return f"run({e.value})"
    elif e.kind == 'read':
        return f"read({e.value})"
    elif e.kind == 'ls':
        return f"ls({e.value})"
    elif e.kind == 'list':
        return '[' + ', '.join(dump_expr(x) for x in e.value) + ']'
    elif e.kind == 'index':
        return f"{dump_expr(e.left)}[{dump_expr(e.right)}]"
    elif e.kind == 'method':
        return f"{dump_expr(e.left)}.{e.value}({', '.join(dump_expr(a) for a in e.args)})"
    return '?'


def dump(node, indent=0):
    pad = "  " * indent
    if isinstance(node, Program):
        print(f"{pad}Program")
        for b in node.blocks:
            dump(b, indent + 1)
    elif isinstance(node, TimeUnit):
        print(f"{pad}TimeUnit [line {node.line}]")
        dump(node.schedule, indent + 1)
        print(f"{pad}  Actions:")
        for a in node.actions:
            dump(a, indent + 2)
        if node.fallback:
            print(f"{pad}  Fallback:")
            for f in node.fallback:
                dump(f, indent + 2)
    elif isinstance(node, ScriptUnit):
        print(f"{pad}ScriptUnit [line {node.line}]")
        print(f"{pad}  Actions:")
        for a in node.actions:
            dump(a, indent + 2)
        if node.fallback:
            print(f"{pad}  Fallback:")
            for f in node.fallback:
                dump(f, indent + 2)
    elif isinstance(node, Schedule):
        print(f"{pad}Schedule({node.kind}, interval={node.interval}, time={node.time})")
    elif isinstance(node, WithUnit):
        print(f"{pad}WithUnit [line {node.line}] config={node.config}")
        dump(node.object, indent + 1)
        print(f"{pad}  Actions:")
        for a in node.actions:
            dump(a, indent + 2)
    elif isinstance(node, ObjectRef):
        print(f"{pad}Object({node.kind}, {node.value})")
    elif isinstance(node, RetryUnit):
        print(f"{pad}RetryUnit({node.times}x) [line {node.line}]")
        for a in node.actions:
            dump(a, indent + 1)
        if node.fallback:
            print(f"{pad}  Fallback:")
            for f in node.fallback:
                dump(f, indent + 2)
    elif isinstance(node, WhenUnit):
        print(f"{pad}WhenUnit({node.condition}) [line {node.line}]")
        print(f"{pad}  Actions:")
        for a in node.actions:
            dump(a, indent + 2)
        if node.fallback:
            print(f"{pad}  Fallback:")
            for f in node.fallback:
                dump(f, indent + 2)
    elif isinstance(node, WatchUnit):
        print(f"{pad}WatchUnit(path={node.path}) [line {node.line}]")
        for a in node.actions:
            dump(a, indent + 1)
    elif isinstance(node, LetStatement):
        print(f"{pad}Let({node.name} = {dump_expr(node.value)}) [line {node.line}]")
    elif isinstance(node, FnDefinition):
        print(f"{pad}Fn({node.name}({', '.join(node.params)})) [line {node.line}]")
        for a in node.body:
            dump(a, indent + 1)
    elif isinstance(node, ForStatement):
        print(f"{pad}For({node.var} in {dump_expr(node.collection)}) [line {node.line}]")
        for a in node.body:
            dump(a, indent + 1)
    elif isinstance(node, ImportStatement):
        print(f"{pad}Import('{node.path}') [line {node.line}]")
    elif isinstance(node, Expr):
        print(f"{pad}{dump_expr(node)} [line {node.line}]")
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
