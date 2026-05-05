"""
E RealDriver — actually executes actions
=========================================

Step 1: Scheduler + Log (working)
Step 2: File + subprocess (working)
Step 3: Browser — open/find/click via Playwright (working)
Step 4+: Stubs with clear "NOT IMPLEMENTED" messages
"""

import os
import time
import subprocess
from pathlib import Path
from typing import Optional

from .base import Driver


class RealDriver(Driver):
    def __init__(self):
        self.scheduler = None
        self._browser = None
        self._pending_jobs = []

    def setup(self):
        try:
            from apscheduler.schedulers.background import BackgroundScheduler
            self.scheduler = BackgroundScheduler()
            self.scheduler.start()
            self.log("  ⚙️ Scheduler started")
        except ImportError:
            self.log("  ⚠️ APScheduler not installed — time blocks will run immediately")
            self.log("    Install: pip install apscheduler")
            self.scheduler = None

    def teardown(self):
        if self.scheduler:
            try:
                self.log("  ⏳ Waiting for scheduled jobs...")
                self.scheduler.shutdown(wait=True)
            except:
                pass

    # ── Scheduling ──

    def schedule_time_block(self, schedule_info: dict, actions_fn, line: int):
        info = f"{schedule_info.get('kind', '?')}"
        if schedule_info.get('interval'):
            info += f" every {schedule_info['interval']}"
        if schedule_info.get('time'):
            info += f" at {schedule_info['time']}"
        self.log(f"⏰ Schedule: {info}", line)

        if self.scheduler is None:
            actions_fn()
            return

        kind = schedule_info.get('kind')
        interval = schedule_info.get('interval')
        time_val = schedule_info.get('time')

        if kind == 'at':
            # Schedule once at given time
            hour, minute = map(int, time_val.split(':'))
            now = time.localtime()
            run_time = time.mktime((now.tm_year, now.tm_mon, now.tm_mday,
                                    hour, minute, 0, now.tm_wday, now.tm_yday, now.tm_isdst))
            if run_time < time.time():
                run_time += 86400  # tomorrow
            self.scheduler.add_job(
                actions_fn,
                'date',
                run_date=time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(run_time)),
                id=f'at_{line}'
            )
            self.log(f"  📅 Scheduled for {time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(run_time))}")

        elif kind == 'every':
            hour, minute = 0, 0
            if time_val:
                hour, minute = map(int, time_val.split(':'))

            interval_map = {
                'minute': lambda: self.scheduler.add_job(
                    actions_fn, 'interval', minutes=1, id=f'every_minute_{line}'),
                'hour': lambda: self.scheduler.add_job(
                    actions_fn, 'interval', hours=1, id=f'every_hour_{line}'),
                'day': lambda: self.scheduler.add_job(
                    actions_fn, 'cron', hour=hour, minute=minute, id=f'every_day_{line}'),
            }
            fn = interval_map.get(interval)
            if fn:
                fn()
                self.log(f"  📅 Scheduled every {interval}" +
                         (f" at {time_val}" if time_val else ""))
            else:
                self.log(f"  ⚠️ Unknown interval '{interval}', running once")
                actions_fn()

    def run_script_block(self, actions_fn, line: int):
        self.log("▶️ Script block", line)
        actions_fn()

    # ── Browser lifecycle ──

    def browser_start(self, line: int):
        if self._browser:
            self.log("  (browser already running)", line)
            return
        try:
            from ..drivers.browser import BrowserDriver
            self._browser = BrowserDriver()
            self._browser.start()
            self.log("  ✅ Browser started", line)
        except RuntimeError as e:
            self.log(f"  ⚠️ {e}", line)
            self._browser = None
        except Exception as e:
            self.log(f"  ⚠️ Failed to start browser: {e}", line)
            self._browser = None

    def browser_stop(self, line: int):
        if self._browser:
            try:
                self._browser.close()
                self.log("  ✅ Browser closed", line)
            except Exception as e:
                self.log(f"  ⚠️ Error closing browser: {e}", line)
            self._browser = None

    # ── Browser actions ──

    def open(self, url: str, line: int):
        if self._browser and self._browser.is_running:
            try:
                self._browser.open(url)
                self.log(f"  🌐 opened '{url}'", line)
                return
            except Exception as e:
                self.log(f"  ⚠️ browser open failed: {e}", line)
        self.log(f"  🌐 open '{url}'", line)
        try:
            import webbrowser
            webbrowser.open(url)
        except Exception as e:
            self.log(f"  ⚠️ Failed to open browser: {e}", line)

    def click(self, selector: Optional[str], line: int):
        if self._browser and self._browser.is_running:
            try:
                self._browser.click(selector)
                self.log(f"  🖱️ clicked '{selector}'", line)
                return
            except Exception as e:
                self.log(f"  ⚠️ browser click failed: {e}", line)
                return
        self.log(f"  🖱️ click '{selector}' (NOT IMPLEMENTED — install Playwright)", line)

    def find(self, selector: str, line: int):
        if self._browser and self._browser.is_running:
            try:
                self._browser.find(selector)
                self.log(f"  🔍 found '{selector}'", line)
                return
            except Exception as e:
                self.log(f"  ⚠️ browser find failed: {e}", line)
                return
        self.log(f"  🔍 find '{selector}' (NOT IMPLEMENTED — install Playwright)", line)

    # ── File ──

    def write(self, path: str, content: str, line: int):
        self.log(f"  ✏️ write '{path}' → {len(content)} chars", line)
        try:
            Path(path).parent.mkdir(parents=True, exist_ok=True)
            with open(path, 'w') as f:
                f.write(content)
            self.log(f"  ✅ wrote to '{path}'", line)
        except Exception as e:
            self.log(f"  ❌ write failed: {e}", line)
            raise

    def create(self, name: str, line: int):
        self.log(f"  🆕 create '{name}'", line)
        try:
            Path(name).touch()
        except Exception as e:
            self.log(f"  ❌ create failed: {e}", line)
            raise

    # ── Transfer ──

    def email(self, to: str, attachment: Optional[str], line: int):
        info = f" (attach: {attachment})" if attachment else ""
        self.log(f"  📧 email to '{to}'{info} (NOT IMPLEMENTED — configure SMTP env vars)", line)

    def upload(self, url: str, file: Optional[str], line: int):
        info = f" ({file})" if file else ""
        self.log(f"  ⬆️ upload to '{url}'{info} (NOT IMPLEMENTED)", line)

    # ── Auth ──

    def login(self, user: str, password: str, line: int):
        self.log(f"  🔐 login '{user}' / '{'*' * len(password)}' (NOT IMPLEMENTED — install Playwright)", line)

    # ── Browser config ──

    def set_page_timeout(self, ms: int, line: int):
        if self._browser:
            self._browser.set_page_timeout(ms)
            self.log(f"  ⏱️ page timeout set to {ms}ms", line)

    # ── Wait ──

    def wait_download(self, line: int):
        self.log(f"  ⏳ wait download...", line)
        time.sleep(2)
        self.log(f"  ✅ download complete (simulated)", line)

    def wait_until(self, condition: str, selector: str, line: int):
        if self._browser and self._browser.is_running:
            try:
                self._browser.wait_until(condition, selector)
                self.log(f"  ⏳ wait until {condition} '{selector}'... ✅", line)
                return
            except Exception as e:
                self.log(f"  ⚠️ wait failed: {e}", line)
                raise RuntimeError(f"element not {condition}: '{selector}'") from e
        self.log(f"  ⏳ wait until {condition} '{selector}'... (simulated)", line)
        time.sleep(1)
        self.log(f"  ✅ condition assumed met (simulated)", line)

    # ── System ──

    def run(self, cmd: str, line: int):
        self.log(f"  ⚡ run '{cmd}'", line)
        try:
            result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
            if result.stdout:
                for line_out in result.stdout.strip().split('\n'):
                    self.log(f"    {line_out}", line)
            if result.stderr:
                for line_err in result.stderr.strip().split('\n'):
                    self.log(f"    ⚠️ {line_err}", line)
            if result.returncode != 0:
                self.log(f"  ❌ exited with code {result.returncode}", line)
        except Exception as e:
            self.log(f"  ❌ run failed: {e}", line)
            raise
