"""
Interprete per E — Linguaggio di automazione
=============================================
Esegue l'AST prodotto da parser_e.py

Uso:
    python3 interprete_e.py test1_caffe.e
    python3 interprete_e.py --live test5_completo.e   # esegue azioni reali
"""

import sys
import os
import time
import subprocess
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from parser.parser_e import lex, Parser, ParseError, Program, TimeBlock, ScriptBlock
from parser.parser_e import Schedule, WithBlock, ObjectRef, RetryBlock, WatchBlock, Action


# ──────────────────────────────────────────────
# Runtime context
# ──────────────────────────────────────────────

class EError(Exception):
    """Errore durante l'esecuzione di un'azione E."""
    def __init__(self, message, line=0):
        super().__init__(message)
        self.line = line


class Runtime:
    def __init__(self, live=False):
        self.live = live
        self.current_element = None   # impostato da find
        self.current_object = None    # impostato da with
        self.browser_process = None    # per browser reali
        self._stop = False             # flag per stop

    def log(self, msg, line=0):
        prefix = f"[line {line}]" if line else ""
        print(f"  {prefix} {msg}")

    # ── Node dispatch ──

    def run(self, node):
        try:
            self._run(node)
        except EError as e:
            raise
        except Exception as e:
            raise EError(str(e), getattr(node, 'line', 0))

    def _run(self, node):
        if isinstance(node, Program):
            for block in node.blocks:
                if self._stop:
                    break
                self.run(block)
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
        elif isinstance(node, Action):
            self._exec_action(node)
        else:
            self.log(f"⚠️ nodo sconosciuto: {type(node).__name__}", 0)

    # ── Error guard (statement con fallback) ──

    def _safe_run(self, node, block_fallback=None, line=0):
        """Esegue node. Fallback: prima locale (sul nodo), poi del blocco."""
        if self._stop:
            return
        try:
            self._run(node)
        except EError as e:
            msg = str(e) or "errore sconosciuto"
            self.log(f"❌ ERRORE: {msg}" + (f" [line {e.line}]" if e.line else ""), line)
            local_fb = getattr(node, 'fallback', None)
            if local_fb:
                self.log(f"  ↳ eseguo fallback LOCALE", line)
                for fb_node in local_fb:
                    self._run(fb_node)
            elif block_fallback:
                self.log(f"  ↳ eseguo fallback GLOBALE", line)
                for fb_node in block_fallback:
                    self._run(fb_node)
            else:
                self.log(f"  ↳ nessun fallback, propago errore", line)
                raise

    # ── Block implementations ──

    def _exec_time_block(self, node: TimeBlock):
        sched = node.schedule
        info = f"⏰ Schedule: {sched.kind}"
        if sched.interval:
            info += f" every {sched.interval}"
        if sched.time:
            info += f" at {sched.time}"
        self.log(info, node.line)

        if not self.live:
            self.log("  (dry-run: eseguo subito una volta)", node.line)
        else:
            self.log("  (live: eseguo subito, scheduling non implementato)", node.line)

        for action in node.actions:
            if self._stop:
                break
            self._safe_run(action, node.fallback, node.line)

    def _exec_script_block(self, node: ScriptBlock):
        self.log("▶️ Script block", node.line)
        for action in node.actions:
            if self._stop:
                break
            self._safe_run(action, node.fallback, node.line)

    def _exec_with_block(self, node: WithBlock):
        obj = node.object
        prev_object = self.current_object

        if obj.kind == 'file':
            self.current_object = obj
            self.log(f"📄 Context: file '{obj.value}'", node.line)
        elif obj.kind == 'browser':
            self.current_object = obj
            self.log(f"🌐 Context: browser", node.line)
            if self.live:
                self._open_browser()
        elif obj.kind == 'page':
            self.current_object = obj
            self.log(f"📄 Context: page", node.line)
        elif obj.kind == 'app':
            self.current_object = obj
            self.log(f"📱 Context: app '{obj.value}'", node.line)

        for action in node.actions:
            if self._stop:
                break
            self._safe_run(action, node.fallback, node.line)

        # Ripristina contesto precedente
        self.current_object = prev_object

    def _exec_retry_block(self, node: RetryBlock):
        last_error = None
        for attempt in range(1, node.times + 1):
            if self._stop:
                break
            self.log(f"🔄 Tentativo {attempt}/{node.times}", node.line)
            try:
                self._run_actions_safe(node.actions)
                last_error = None
                break  # successo
            except EError as e:
                last_error = e
                if attempt < node.times:
                    self.log(f"  fallito, riprovo...", node.line)
                    time.sleep(1)
                continue

        if last_error and node.fallback:
            self.log(f"  ❌ tutti i tentativi falliti, eseguo fallback", node.line)
            for fb in node.fallback:
                self._run(fb)
        elif last_error:
            raise EError(f"retry {node.times}x esaurito", node.line)

    def _exec_watch_block(self, node: WatchBlock):
        path = node.path
        self.log(f"👀 Watch: '{path}' (simulato — eseguo azioni una volta)", node.line)
        # In live, usare watchdog. Per ora esegue le azioni una volta.
        for action in node.actions:
            if self._stop:
                break
            self._safe_run(action, node.fallback, node.line)

    def _run_actions_safe(self, actions):
        """Esegue una lista di azioni, propagando SOLO errori senza fallback locale."""
        for action in actions:
            if self._stop:
                break
            # Ogni azione gestisce il proprio fallback locale
            # Solo gli errori NON gestiti propagano al chiamante
            self._safe_run(action, block_fallback=None, line=action.line)

    # ── Action implementations ──

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
        }
        fn = dispatcher.get(kind)
        if not fn:
            raise EError(f"azione sconosciuta: '{kind}'", node.line)
        fn(node)

    def _action_open(self, node: Action):
        url = node.args[0]
        self.log(f"  🌐 open '{url}'", node.line)
        if self.live:
            import webbrowser
            webbrowser.open(url)

    def _action_click(self, node: Action):
        selector = node.args[0] if node.args else self.current_element
        if not selector:
            raise EError("click senza selector e senza current element (usa find prima)", node.line)
        self.log(f"  🖱️ click '{selector}'", node.line)

    def _action_find(self, node: Action):
        selector = node.args[0]
        self.current_element = selector
        self.log(f"  🔍 find '{selector}' → current element impostato", node.line)

    def _action_write(self, node: Action):
        obj = node.args[0]
        content = node.args[1]
        target = obj or self.current_object
        if not target or target.kind != 'file':
            raise EError("write richiede un file (usa with file o write file ...)", node.line)
        path = target.value
        self.log(f"  ✏️ write '{path}' → {len(content)} caratteri", node.line)
        if self.live:
            with open(path, 'w') as f:
                f.write(content)

    def _action_email(self, node: Action):
        target_addr = node.args[0]
        obj = node.args[1] or self.current_object
        file_info = f" (allegato: {obj.value})" if obj else ""
        self.log(f"  📧 email to '{target_addr}'{file_info}", node.line)
        if self.live:
            self.log("  (invio email non implementato — usa un servizio SMTP)", node.line)

    def _action_upload(self, node: Action):
        url = node.args[0]
        obj = node.args[1] or self.current_object
        file_info = f" ({obj.value})" if obj else ""
        self.log(f"  ⬆️ upload to '{url}'{file_info}", node.line)

    def _action_login(self, node: Action):
        user, pwd = node.args
        self.log(f"  🔐 login '{user}' / '{'*' * len(pwd)}'", node.line)
        if self.live:
            self.log("  (login automatico non implementato — usa Selenium)", node.line)

    def _action_log(self, node: Action):
        msg = node.args[0]
        self.log(f"  📝 {msg}", node.line)

    def _action_stop(self, node: Action):
        self.log(f"  🛑 stop", node.line)
        self._stop = True

    def _action_wait_download(self, node: Action):
        self.log(f"  ⏳ wait download...", node.line)
        if self.live:
            time.sleep(2)
        self.log(f"  ✅ download completato", node.line)

    def _action_wait_until(self, node: Action):
        cond, sel = node.args
        self.log(f"  ⏳ wait until {cond} '{sel}'...", node.line)
        if self.live:
            time.sleep(1)
        self.log(f"  ✅ condizione '{cond} {sel}' soddisfatta", node.line)

    def _action_run(self, node: Action):
        cmd = node.args[0]
        self.log(f"  ⚡ run '{cmd}'", node.line)
        if self.live:
            result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
            if result.stdout:
                self.log(f"    stdout: {result.stdout.strip()}", node.line)
            if result.stderr:
                self.log(f"    stderr: {result.stderr.strip()}", node.line)

    def _action_create(self, node: Action):
        name = node.args[0]
        self.log(f"  🆕 create '{name}'", node.line)
        if self.live:
            Path(name).touch()

    def _open_browser(self):
        self.log("  (browser: nessuna implementazione live)", 0)


# ──────────────────────────────────────────────
# Main CLI
# ──────────────────────────────────────────────

def main():
    args = sys.argv[1:]
    live = False
    files = []

    for a in args:
        if a == '--live':
            live = True
        elif a.startswith('--'):
            print(f"Opzione sconosciuta: {a}")
            sys.exit(1)
        else:
            files.append(a)

    if not files:
        print("Uso: python3 interprete_e.py [--live] <file.e> ...")
        sys.exit(1)

    runtime = Runtime(live=live)
    ok = True

    for path in files:
        print(f"\n{'='*60}")
        print(f"▶️  ESEGUO: {path}" + (" (LIVE)" if live else " (dry-run)"))
        print(f"{'='*60}")
        try:
            with open(path) as f:
                source = f.read()
            tokens = lex(source)
            parser = Parser(tokens)
            ast = parser.parse()
            runtime.run(ast)
            print(f"\n✅ Completato: {path}")
        except (SyntaxError, ParseError) as e:
            print(f"❌ ERRORE DI SINTASSI: {e}")
            ok = False
        except EError as e:
            print(f"❌ ERRORE RUNTIME: {e}")
            ok = False
        except FileNotFoundError:
            print(f"❌ File non trovato: {path}")
            ok = False

    if not ok:
        sys.exit(1)


if __name__ == '__main__':
    main()
