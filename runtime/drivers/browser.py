"""
E — Browser Driver (Playwright wrapper)
=========================================
Surface: open, find, click, wait_until, login, download.

Usage:
    b = BrowserDriver()
    b.open("https://example.com/login")
    b.login("user", "pass")
    b.wait_download()
    filepath = b.last_download
    b.close()
"""

import os
import time
from pathlib import Path
from typing import Optional


class BrowserDriver:
    def __init__(self):
        self._playwright = None
        self._browser = None
        self._context = None
        self._page = None
        self._current_element = None
        self._page_timeout = 10000
        self._download_dir = None
        self.last_download = None

    # ── Lifecycle ──

    def start(self, download_dir: str = None):
        try:
            from playwright.sync_api import sync_playwright
            self._playwright = sync_playwright().__enter__()
            self._browser = self._playwright.chromium.launch(headless=False)

            if download_dir:
                self._download_dir = download_dir
                Path(download_dir).mkdir(parents=True, exist_ok=True)
                self._context = self._browser.new_context(
                    accept_downloads=True
                )
            else:
                self._context = self._browser.new_context(
                    accept_downloads=True
                )
            self._page = None
        except ImportError:
            raise RuntimeError(
                "Playwright not installed.\n"
                "  pip install playwright\n"
                "  playwright install"
            )

    def close(self):
        try:
            if self._browser:
                self._browser.close()
            if self._playwright:
                self._playwright.__exit__(None, None, None)
        except:
            pass
        self._playwright = None
        self._browser = None
        self._context = None
        self._page = None
        self._current_element = None
        self.last_download = None

    def set_page_timeout(self, ms: int):
        self._page_timeout = ms

    # ── Actions ──

    def open(self, url: str):
        if not self._browser:
            self.start()
        self._page = self._context.new_page()
        self._page.goto(url)
        self._current_element = None

    def find(self, selector: str):
        if not self._page:
            raise RuntimeError("No page open. Use open() first.")
        self._page.wait_for_selector(selector, state="attached", timeout=self._page_timeout)
        self._current_element = selector
        return selector

    def click(self, selector: Optional[str] = None):
        target = selector or self._current_element
        if not target:
            raise RuntimeError("No selector and no current element. Use find() first.")
        if not self._page:
            raise RuntimeError("No page open. Use open() first.")
        loc = self._page.locator(target)
        loc.wait_for(state="visible", timeout=self._page_timeout)
        loc.click()

    def login(self, username: str, password: str):
        if not self._page:
            raise RuntimeError("No page open. Use open() first.")

        # Auto-detect login form via common selectors
        selectors = {
            'username': [
                'input[type="email"]', 'input[name="email"]',
                'input[type="text"][name="username"]', 'input[name="login"]',
                'input[name="user"]', '#email', '#username', '#login',
                'input[autocomplete="username"]', 'input[type="text"]:not([name=""])',
            ],
            'password': [
                'input[type="password"]', '#password', 'input[name="password"]',
                'input[autocomplete="current-password"]',
            ],
            'submit': [
                'button[type="submit"]', 'input[type="submit"]',
                'button:has-text("Log in")', 'button:has-text("Sign in")',
                'button:has-text("Login")', 'button:has-text("Accedi")',
                'button:has-text("Entra")', 'button:has-text("Submit")',
            ],
        }

        user_field = None
        for sel in selectors['username']:
            try:
                loc = self._page.locator(sel)
                if loc.count() > 0:
                    user_field = loc.first
                    break
            except:
                continue

        pass_field = None
        for sel in selectors['password']:
            try:
                loc = self._page.locator(sel)
                if loc.count() > 0:
                    pass_field = loc.first
                    break
            except:
                continue

        if not user_field or not pass_field:
            raise RuntimeError("Could not auto-detect login form. Use find + click manually.")

        user_field.fill(username)
        pass_field.fill(password)

        for sel in selectors['submit']:
            try:
                loc = self._page.locator(sel)
                if loc.count() > 0:
                    loc.first.click()
                    return
            except:
                continue

        # fallback: press Enter on password field
        pass_field.press("Enter")

    def wait_download(self, timeout: int = 30):
        if not self._page:
            raise RuntimeError("No page open. Use open() first.")

        with self._page.expect_download(timeout=timeout * 1000) as download_info:
            pass  # wait for download to start

        download = download_info.value

        if self._download_dir:
            path = os.path.join(self._download_dir, download.suggested_filename)
            download.save_as(path)
        else:
            path = download.suggested_filename
            download.save_as(path)

        self.last_download = path

    def find_all(self, selector: str):
        if not self._page:
            raise RuntimeError("No page open. Use open() first.")
        return self._page.locator(selector).all()

    def get_number(self, selector: str) -> Optional[float]:
        if not self._page:
            raise RuntimeError("No page open. Use open() first.")
        text = self._page.locator(selector).text_content(timeout=5000)
        if text:
            import re
            nums = re.findall(r'[\d.]+', text)
            if nums:
                return float(nums[0].replace(',', ''))
        return None

    def wait_until(self, condition: str, selector: str):
        if not self._page:
            raise RuntimeError("No page open. Use open() first.")
        state_map = {
            "visible": "visible",
            "hidden": "hidden",
        }
        state = state_map.get(condition)
        if not state:
            raise ValueError(f"Unknown condition: '{condition}'. Use 'visible' or 'hidden'.")
        loc = self._page.locator(selector)
        loc.wait_for(state=state, timeout=self._page_timeout)

    @property
    def page(self):
        return self._page

    @property
    def is_running(self):
        return self._browser is not None and self._browser.is_connected()
