"""
E — Executor: walks AST, calls driver, handles context/retry/fallback
"""

from .context import RuntimeContext
from .drivers.base import Driver, DryDriver
from .drivers.real import RealDriver
from parser.main import (
    Program, TimeNode, ScriptNode, Schedule,
    WithNode, ObjectRef, RetryNode, WatchNode, WhenNode, Action,
    LetStatement, FnDefinition, ForStatement, UseStatement,
    Expr, dump_expr,
)


class EError(Exception):
    def __init__(self, message, line=0):
        super().__init__(message)
        self.line = line


class Executor:
    def __init__(self, driver: Driver = None, live=False):
        self.driver = driver or (RealDriver() if live else DryDriver())
        self.ctx = RuntimeContext()
        self._watchers = []

    def run(self, node):
        self.driver.setup()
        try:
            self._run(node)
            if self.ctx.should_stop:
                self.driver.log("🛑 Execution stopped", 0)
        finally:
            for w in self._watchers:
                try:
                    w.stop()
                except:
                    pass
            self.driver.teardown()

    def _run(self, node):
        if isinstance(node, Program):
            for block in node.blocks:
                if self.ctx.should_stop:
                    break
                self._run(block)

        elif isinstance(node, TimeNode):
            self._exec_time_block(node)

        elif isinstance(node, ScriptNode):
            self._exec_script_block(node)

        elif isinstance(node, WithNode):
            self._exec_with_block(node)

        elif isinstance(node, RetryNode):
            self._exec_retry_block(node)

        elif isinstance(node, WatchNode):
            self._exec_watch_block(node)

        elif isinstance(node, WhenNode):
            self._exec_when_block(node)

        elif isinstance(node, Action):
            self._exec_action(node)

        elif isinstance(node, LetStatement):
            val = self._eval_expr(node.value)
            self.ctx.scope.def_var(node.name, val)
            self.driver.log(f"  📦 let {node.name} = {val}", node.line)

        elif isinstance(node, FnDefinition):
            self.ctx.scope.def_fn(node.name, node)
            self.driver.log(f"  📦 fn {node.name}({', '.join(node.params)})", node.line)

        elif isinstance(node, ForStatement):
            self._exec_for(node)

        elif isinstance(node, UseStatement):
            self._exec_import(node)

        elif isinstance(node, Expr):
            result = self._eval_expr(node)
            if result is not None:
                # Return value for function body
                self._expr_result = result

        else:
            self.driver.log(f"⚠️ Unknown node: {type(node).__name__}", 0)

    # ── Error guard ──

    def _safe(self, node, unit_fallback=None):
        """Execute node with fallback chain: local → block."""
        if self.ctx.should_stop:
            return
        try:
            self._run(node)
        except EError as e:
            msg = str(e) or "error"
            self.driver.log(f"  ❌ ERROR: {msg}" + (f" [line {e.line}]" if e.line else ""),
                            getattr(node, 'line', 0))
            local_fb = getattr(node, 'fallback', None)
            if local_fb:
                self.driver.log(f"  ↳ running LOCAL fallback", getattr(node, 'line', 0))
                for fb in local_fb:
                    self._run(fb)
            elif unit_fallback:
                self.driver.log(f"  ↳ running UNIT fallback", getattr(node, 'line', 0))
                for fb in unit_fallback:
                    self._run(fb)
            else:
                raise

    # ── Block executors ──

    def _exec_time_block(self, node: TimeNode):
        sched = node.schedule
        info = {
            'kind': sched.kind,
            'interval': sched.interval,
            'time': sched.time,
        }

        def actions_fn():
            for action in node.actions:
                if self.ctx.should_stop:
                    break
                self._safe(action, node.fallback)

        self.driver.schedule_time_block(info, actions_fn, node.line)

    def _exec_script_block(self, node: ScriptNode):
        def actions_fn():
            for action in node.actions:
                if self.ctx.should_stop:
                    break
                self._safe(action, node.fallback)

        self.driver.run_script_block(actions_fn, node.line)

    def _exec_with_block(self, node: WithNode):
        self.ctx.push_object(node.object)
        try:
            if node.object.kind == 'file':
                self.driver.log(f"📄 Context: file '{node.object.value}'", node.line)
            elif node.object.kind == 'browser':
                self.driver.log(f"🌐 Context: browser", node.line)
                self.driver.browser_start(node.line)
            elif node.object.kind == 'page':
                self.driver.log(f"📄 Context: page", node.line)
                if node.config:
                    ms = self._parse_timeout(node.config)
                    self.driver.set_page_timeout(ms, node.line)
            elif node.object.kind == 'app':
                self.driver.log(f"📱 Context: app '{node.object.value}'", node.line)

            for action in node.actions:
                if self.ctx.should_stop:
                    break
                self._safe(action, node.fallback)
        finally:
            if node.object.kind == 'browser':
                self.driver.browser_stop(node.line)
            self.ctx.pop_object()

    def _exec_retry_block(self, node: RetryNode):
        last_error = None
        for attempt in range(1, node.times + 1):
            if self.ctx.should_stop:
                break
            self.driver.log(f"🔄 Attempt {attempt}/{node.times}", node.line)
            try:
                self._exec_action_list_safe(node.actions)
                last_error = None
                break
            except EError as e:
                last_error = e
                if attempt < node.times:
                    self.driver.log(f"  failed, retrying...", node.line)
                    import time
                    time.sleep(1)

        if last_error and node.fallback:
            self.driver.log(f"  ❌ all attempts failed, running fallback", node.line)
            for fb in node.fallback:
                self._run(fb)
        elif last_error:
            raise EError(f"retry {node.times}x exhausted", node.line)

    def _exec_watch_block(self, node: WatchNode):
        path = node.path
        self.driver.log(f"👀 Watch: '{path}'", node.line)

        try:
            from watchdog.observers import Observer
            from watchdog.events import FileSystemEventHandler

            class Handler(FileSystemEventHandler):
                def __init__(self, exe, acts, fb, ln):
                    self.exe = exe
                    self.acts = acts
                    self.fb = fb
                    self.ln = ln
                def on_created(self, event):
                    if event.is_directory:
                        return
                    self.exe.driver.log(f"  📄 new: {event.src_path}", self.ln)
                    for a in self.acts:
                        if self.exe.ctx.should_stop:
                            break
                        self.exe._safe(a, self.fb)
                def on_modified(self, event):
                    if event.is_directory:
                        return
                    for a in self.acts:
                        if self.exe.ctx.should_stop:
                            break
                        self.exe._safe(a, self.fb)

            observer = Observer()
            observer.schedule(Handler(self, node.actions, node.fallback, node.line), path)
            observer.start()
            self._watchers.append(observer)
            self.driver.log(f"  ✅ watching '{path}'", node.line)

        except ImportError:
            self.driver.log(f"  ⚠️ watchdog not installed — running once", node.line)
            for action in node.actions:
                if self.ctx.should_stop:
                    break
                self._safe(action, node.fallback)
        except Exception as e:
            self.driver.log(f"  ⚠️ watch failed: {e}", node.line)

    @staticmethod
    def _parse_timeout(config: str) -> int:
        if config.endswith('ms'):
            return int(config[:-2])
        if config.endswith('s'):
            return int(config[:-1]) * 1000
        return int(config) * 1000

    def _exec_when_block(self, node: WhenNode):
        cond = node.condition
        if cond.get('type') == 'expr':
            # General expression condition
            result = self._eval_expr(cond['expr'])
            self.driver.log(f"  🔍 when ({dump_expr(cond['expr'])}) → {bool(result)}", node.line)
        else:
            # Semantic condition (item visible, count > 5, etc.)
            result = self.driver.evaluate_condition(cond, self.ctx, node.line)
        if result:
            self.driver.log(f"  ➡️ condition true, executing unit", node.line)
            for action in node.actions:
                if self.ctx.should_stop:
                    break
                self._safe(action, node.fallback)
        else:
            self.driver.log(f"  ➡️ condition false, skipping unit", node.line)

    # ── Expression evaluation ──

    def _eval_expr(self, expr: Expr):
        if expr.kind == 'num':
            return expr.value
        elif expr.kind == 'str':
            return expr.value
        elif expr.kind == 'run':
            import subprocess
            cmd = expr.value
            stdin_data = None
            if expr.args and expr.args[0] is not None:
                stdin_val = self._eval_expr(expr.args[0])
                stdin_data = str(stdin_val) if stdin_val else None
            result = subprocess.run(cmd, shell=True, capture_output=True, text=True,
                                     input=stdin_data)
            return result.stdout.rstrip('\n')

        elif expr.kind == 'read':
            path = expr.value
            try:
                with open(path) as f:
                    return f.read()
            except Exception as e:
                raise EError(f"cannot read '{path}': {e}", expr.line)

        elif expr.kind == 'ls':
            from pathlib import Path
            pattern = expr.value
            files = [str(p) for p in sorted(Path().glob(pattern)) if p.is_file()]
            return '\n'.join(files)

        elif expr.kind == 'list':
            return [self._eval_expr(item) for item in expr.value]

        elif expr.kind == 'index':
            container = self._eval_expr(expr.left)
            idx = self._eval_expr(expr.right)
            return container[idx]

        elif expr.kind == 'method':
            obj = self._eval_expr(expr.left)
            method = expr.value
            args = [self._eval_expr(a) for a in expr.args]
            if method == 'append' and isinstance(obj, list):
                obj.append(args[0])
                return obj
            raise EError(f"unknown method '{method}'", expr.line)

        elif expr.kind == 'var':
            try:
                return self.ctx.scope.get_var(expr.value)
            except NameError as e:
                raise EError(str(e), expr.line)
        elif expr.kind == 'call':
            try:
                fn_def = self.ctx.scope.get_fn(expr.value)
            except NameError as e:
                raise EError(str(e), expr.line)

            if len(expr.args) != len(fn_def.params):
                raise EError(
                    f"function '{expr.value}' expects {len(fn_def.params)} args, got {len(expr.args)}",
                    expr.line)

            # Evaluate arguments
            arg_vals = [self._eval_expr(a) for a in expr.args]

            # Create new scope and bind params
            self.ctx.push_scope()
            for param, val in zip(fn_def.params, arg_vals):
                self.ctx.scope.def_var(param, val)

            # Execute function body
            result = None
            self._expr_result = None
            for action in fn_def.body:
                if self.ctx.should_stop:
                    break
                self._run(action)
            result = self._expr_result
            self._expr_result = None

            self.ctx.pop_scope()
            return result

        elif expr.kind == 'bin':
            left = self._eval_expr(expr.left)
            right = self._eval_expr(expr.right)
            op = expr.op
            if op == '+':
                if isinstance(left, str) or isinstance(right, str):
                    return str(left) + str(right)
                return left + right
            elif op == '-':
                return left - right
            elif op == '*':
                return left * right
            elif op == '/':
                return left / right
            elif op == '>':
                return left > right
            elif op == '<':
                return left < right
            elif op == '>=':
                return left >= right
            elif op == '<=':
                return left <= right
            elif op == '==':
                return left == right
            elif op == '!=':
                return left != right
            elif op == 'and':
                return left and right
            elif op == 'or':
                return left or right
            else:
                raise EError(f"unknown operator '{op}'", expr.line)

        return None

    # ── For loop ──

    def _exec_for(self, node: ForStatement):
        collection = self._eval_expr(node.collection)
        if isinstance(collection, str):
            collection = collection.split('\n')
        if not isinstance(collection, (list, tuple)):
            collection = [collection]

        for item in collection:
            if self.ctx.should_stop:
                break
            self.ctx.scope.def_var(node.var, item)
            for action in node.body:
                if self.ctx.should_stop:
                    break
                self._safe(action, node.fallback)

    # ── Import ──

    def _exec_import(self, node: UseStatement):
        path = node.path
        if not path.endswith('.e'):
            path += '.e'
        try:
            with open(path) as f:
                source = f.read()
        except FileNotFoundError:
            raise EError(f"module not found: '{path}'", node.line)
        from parser.main import lex, Parser
        tokens = lex(source)
        parser = Parser(tokens)
        module_ast = parser.parse()
        # Execute all script blocks in the module (top-level code + function defs)
        for block in module_ast.blocks:
            self._run(block)

    def _exec_action_list_safe(self, actions):
        """Execute actions, propagating only errors without local fallback."""
        for action in actions:
            if self.ctx.should_stop:
                break
            self._safe(action, unit_fallback=None)

    # ── Action dispatch ──

    def _exec_action(self, node: Action):
        kind = node.kind
        dispatcher = {
            'open': self._action_open,
            'click': self._action_click,
            'find': self._action_find,
            'write': self._action_write,
            'email': self._action_email,
            'upload': self._action_upload,
            'login': self._action_login,
            'log': self._action_log,
            'log_expr': self._action_log_expr,
            'stop': self._action_stop,
            'wait_download': self._action_wait_download,
            'wait_until': self._action_wait_until,
            'run': self._action_run,
            'create': self._action_create,
            'get_number': self._action_get_number,
            'find_all': self._action_find_all,
        }
        fn = dispatcher.get(kind)
        if not fn:
            raise EError(f"unknown action: '{kind}'", node.line)
        fn(node)

    # ── Action handlers ──

    def _action_open(self, node: Action):
        self.driver.open(node.args[0], node.line)

    def _action_click(self, node: Action):
        selector = node.args[0] if node.args else self.ctx.current_element
        if not selector:
            raise EError("click needs a selector or a current element (use find first)", node.line)
        self.driver.click(selector, node.line)

    def _action_find(self, node: Action):
        self.ctx.current_element = node.args[0]
        self.driver.find(node.args[0], node.line)

    def _action_write(self, node: Action):
        obj = node.args[0]
        content = node.args[1]
        target = obj or self.ctx.current_object
        if not target or target.kind != 'file':
            raise EError("write needs a file (use with file or write file ...)", node.line)
        self.driver.write(target.value, content, node.line)

    def _action_email(self, node: Action):
        to = node.args[0]
        obj = node.args[1] or self.ctx.current_object
        attachment = obj.value if obj else None
        self.driver.email(to, attachment, node.line)

    def _action_upload(self, node: Action):
        url = node.args[0]
        obj = node.args[1] or self.ctx.current_object
        file_path = obj.value if obj else None
        self.driver.upload(url, file_path, node.line)

    def _action_login(self, node: Action):
        user, pwd = node.args
        self.driver.login(user, pwd, node.line)

    def _action_log(self, node: Action):
        self.driver.log(f"  📝 {node.args[0]}", node.line)

    def _action_log_expr(self, node: Action):
        val = self._eval_expr(node.args[0])
        self.driver.log(f"  📝 {val}", node.line)

    def _action_stop(self, node: Action):
        self.driver.log(f"  🛑 stop", node.line)
        self.ctx.stop()

    def _action_wait_download(self, node: Action):
        self.driver.wait_download(node.line)

    def _action_wait_until(self, node: Action):
        cond, sel = node.args
        self.driver.wait_until(cond, sel, node.line)

    def _action_run(self, node: Action):
        out = self._eval_expr(Expr('run', node.args[0], line=node.line))
        if out:
            for line_out in out.split('\n'):
                self.driver.log(f"  {line_out}", node.line)

    def _action_create(self, node: Action):
        self.driver.create(node.args[0], node.line)

    def _action_get_number(self, node: Action):
        selector = node.args[0]
        self.driver.get_number(selector, self.ctx, node.line)

    def _action_find_all(self, node: Action):
        selector = node.args[0]
        self.driver.find_all(selector, self.ctx, node.line)
