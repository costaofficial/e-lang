"""
E Driver base class + Dry implementation
"""

from abc import ABC, abstractmethod
from typing import Optional


class Driver(ABC):
    """Abstract driver. All methods get line number for error reporting."""

    def log(self, msg: str, line: int = 0):
        prefix = f"[line {line}]" if line else ""
        print(f"  {prefix} {msg}")

    # ── Lifecycle ──

    @abstractmethod
    def setup(self):
        """Called once before execution."""

    @abstractmethod
    def teardown(self):
        """Called once after execution."""

    # ── Browser actions ──

    @abstractmethod
    def open(self, url: str, line: int):
        ...

    @abstractmethod
    def click(self, selector: Optional[str], line: int):
        ...

    @abstractmethod
    def find(self, selector: str, line: int):
        ...

    # ── File actions ──

    @abstractmethod
    def write(self, path: str, content: str, line: int):
        ...

    @abstractmethod
    def create(self, name: str, line: int):
        ...

    # ── Transfer actions ──

    @abstractmethod
    def email(self, to: str, attachment: Optional[str], line: int):
        ...

    @abstractmethod
    def upload(self, url: str, file: Optional[str], line: int):
        ...

    # ── Auth ──

    @abstractmethod
    def login(self, user: str, password: str, line: int):
        ...

    # ── Wait ──

    @abstractmethod
    def wait_download(self, line: int):
        ...

    @abstractmethod
    def wait_until(self, condition: str, selector: str, line: int):
        ...

    # ── System ──

    @abstractmethod
    def run(self, cmd: str, line: int):
        ...

    # ── Scheduling ──

    @abstractmethod
    def schedule_time_block(self, schedule_info: dict, actions_fn, line: int):
        """Schedule a time block for execution. actions_fn is a callable."""
        ...

    @abstractmethod
    def run_script_block(self, actions_fn, line: int):
        """Run a script block immediately."""
        ...


class DryDriver(Driver):
    """Dry-run driver: logs everything, does nothing."""

    def setup(self):
        pass

    def teardown(self):
        pass

    def open(self, url: str, line: int):
        self.log(f"  🌐 open '{url}'", line)

    def click(self, selector: Optional[str], line: int):
        self.log(f"  🖱️ click '{selector}'" if selector else "  🖱️ click (current element)", line)

    def find(self, selector: str, line: int):
        self.log(f"  🔍 find '{selector}'", line)

    def write(self, path: str, content: str, line: int):
        self.log(f"  ✏️ write '{path}' → {len(content)} chars", line)

    def create(self, name: str, line: int):
        self.log(f"  🆕 create '{name}'", line)

    def email(self, to: str, attachment: Optional[str], line: int):
        info = f" (attach: {attachment})" if attachment else ""
        self.log(f"  📧 email to '{to}'{info}", line)

    def upload(self, url: str, file: Optional[str], line: int):
        info = f" ({file})" if file else ""
        self.log(f"  ⬆️ upload to '{url}'{info}", line)

    def login(self, user: str, password: str, line: int):
        self.log(f"  🔐 login '{user}' / '{'*' * len(password)}'", line)

    def wait_download(self, line: int):
        self.log(f"  ⏳ wait download...", line)
        self.log(f"  ✅ download complete", line)

    def wait_until(self, condition: str, selector: str, line: int):
        self.log(f"  ⏳ wait until {condition} '{selector}'...", line)
        self.log(f"  ✅ condition met", line)

    def run(self, cmd: str, line: int):
        self.log(f"  ⚡ run '{cmd}'", line)

    def schedule_time_block(self, schedule_info: dict, actions_fn, line: int):
        info = f"every {schedule_info['interval']}" if schedule_info.get('interval') else ""
        if schedule_info.get('time'):
            info += f" at {schedule_info['time']}"
        self.log(f"⏰ Schedule: {info} (dry-run: running once)", line)
        actions_fn()

    def run_script_block(self, actions_fn, line: int):
        self.log("▶️ Script block (dry-run)", line)
        actions_fn()
