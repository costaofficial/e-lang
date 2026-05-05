"""
E — Browser Driver (Playwright wrapper)
=========================================
Surface: open, find, click, wait_until.

Usage:
    b = BrowserDriver()
    b.open("https://example.com")
    b.find("#login")
    b.click()
    b.wait_until("visible", "#chart")
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
        self._page_timeout = 10000

    # ── Lifecycle ──

    def start(self):
        try:
            from playwright.sync_api import sync_playwright
            self._playwright = sync_playwright().__enter__()
            self._browser = self._playwright.chromium.launch(headless=False)
            self._context = self._browser.new_context()
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
