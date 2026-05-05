"""
E — Executor: walks AST, calls driver, handles context/retry/fallback
"""

from .context import RuntimeContext
from .drivers.base import Driver, DryDriver
from .drivers.real import RealDriver
from parser.parser_e import (
    Program, TimeBlock, ScriptBlock, Schedule,
    WithBlock, ObjectRef, RetryBlock, WatchBlock, WhenBlock, Action
)


class EError(Exception):
    def __init__(self, message, line=0):
        super().__init__(message)
        self.line = line


class Executor:
    def __init__(self, driver: Driver = None, live=False):
        self.driver = driver or (RealDriver() if live else DryDriver())
        self.ctx = RuntimeContext()

    def run(self, node):
        self.driver.setup()
        try:
            self._run(node)
            if self.ctx.should_stop:
                self.driver.log("🛑 Execution stopped", 0)
        finally:
            self.driver.teardown()

    def _run(self, node):
        if isinstance(node, Program):
            for block in node.blocks:
                if self.ctx.should_stop:
                    break
                self._run(block)

        elif isinstance(node, TimeBlock):
            self._exec_time_block(node)

        elif isinstance(node, ScriptBlock):
            self._exec_script_block(node)

        elif isinstance(node, WithBlock):
            self._exec_with_block(node)

        elif isinstance(node, RetryBlock):
            self._exec_retry_block(node)

        elif isinstance(node, WatchBlock):
            self._exec_watch_block(node)

        elif isinstance(node, WhenBlock):
            self._exec_when_block(node)

        elif isinstance(node, Action):
            self._exec_action(node)

        else:
            self.driver.log(f"⚠️ Unknown node: {type(node).__name__}", 0)

    # ── Error guard ──

    def _safe(self, node, block_fallback=None):
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
            elif block_fallback:
                self.driver.log(f"  ↳ running BLOCK fallback", getattr(node, 'line', 0))
                for fb in block_fallback:
                    self._run(fb)
            else:
                raise

    # ── Block executors ──

    def _exec_time_block(self, node: TimeBlock):
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

    def _exec_script_block(self, node: ScriptBlock):
        def actions_fn():
            for action in node.actions:
                if self.ctx.should_stop:
                    break
                self._safe(action, node.fallback)

        self.driver.run_script_block(actions_fn, node.line)

    def _exec_with_block(self, node: WithBlock):
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

    def _exec_retry_block(self, node: RetryBlock):
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

    def _exec_watch_block(self, node: WatchBlock):
        self.driver.log(f"👀 Watch: '{node.path}' (simulated — running once)", node.line)
        for action in node.actions:
            if self.ctx.should_stop:
                break
            self._safe(action, node.fallback)

    @staticmethod
    def _parse_timeout(config: str) -> int:
        if config.endswith('ms'):
            return int(config[:-2])
        if config.endswith('s'):
            return int(config[:-1]) * 1000
        return int(config) * 1000

    def _exec_when_block(self, node: WhenBlock):
        result = self.driver.evaluate_condition(node.condition, self.ctx, node.line)
        if result:
            self.driver.log(f"  ➡️ condition true, executing block", node.line)
            for action in node.actions:
                if self.ctx.should_stop:
                    break
                self._safe(action, node.fallback)
        else:
            self.driver.log(f"  ➡️ condition false, skipping block", node.line)

    def _exec_action_list_safe(self, actions):
        """Execute actions, propagating only errors without local fallback."""
        for action in actions:
            if self.ctx.should_stop:
                break
            self._safe(action, block_fallback=None)

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

    def _action_stop(self, node: Action):
        self.driver.log(f"  🛑 stop", node.line)
        self.ctx.stop()

    def _action_wait_download(self, node: Action):
        self.driver.wait_download(node.line)

    def _action_wait_until(self, node: Action):
        cond, sel = node.args
        self.driver.wait_until(cond, sel, node.line)

    def _action_run(self, node: Action):
        self.driver.run(node.args[0], node.line)

    def _action_create(self, node: Action):
        self.driver.create(node.args[0], node.line)

    def _action_get_number(self, node: Action):
        selector = node.args[0]
        self.driver.get_number(selector, self.ctx, node.line)

    def _action_find_all(self, node: Action):
        selector = node.args[0]
        self.driver.find_all(selector, self.ctx, node.line)
