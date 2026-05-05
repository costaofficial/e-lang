"""
E Browser Driver — Playwright wrapper
======================================
Minimal surface: open, find, click.

Usage:
    from .browser import BrowserDriver
    b = BrowserDriver()
    b.open("https://example.com")
    b.find("#login")
    b.click()
    b.close()
"""

from typing import Optional


class BrowserDriver:
    def __init__(self):
        self._playwright = None
        self._browser = None
        self._context = None
        self._page = None
        self._current_element = None

    # ── Lifecycle ──

    def start(self):
        try:
            from playwright.sync_api import sync_playwright
            self._playwright = sync_playwright().__enter__()
            self._browser = self._playwright.chromium.launch(headless=False)
            self._context = self._browser.new_context()
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
        self._page.wait_for_selector(selector, state="attached", timeout=10000)
        self._current_element = selector
        return selector

    def click(self, selector: Optional[str] = None):
        target = selector or self._current_element
        if not target:
            raise RuntimeError("No selector and no current element. Use find() first.")
        if not self._page:
            raise RuntimeError("No page open. Use open() first.")
        self._page.click(target)

    @property
    def page(self):
        return self._page

    @property
    def is_running(self):
        return self._browser is not None and self._browser.is_connected()
