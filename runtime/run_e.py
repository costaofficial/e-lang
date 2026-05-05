"""
E — general-purpose language
=============================
CLI entry point.

Usage:
    python3 runtime/run_e.py script.e                 # dry-run
    python3 runtime/run_e.py --live script.e          # live execution
    python3 runtime/run_e.py --live --watch script.e  # keep alive for scheduler
"""

import sys
import time
from pathlib import Path

# Make local development work without pip install
if __name__ == '__main__' or not __package__:
    root = str(Path(__file__).resolve().parent.parent)
    if root not in sys.path:
        sys.path.insert(0, root)

from parser.parser_e import lex, Parser, ParseError
from runtime.executor import Executor, EError


# ──────────────────────────────────────────────
# Main CLI
# ──────────────────────────────────────────────

def main():
    args = sys.argv[1:]
    live = False
    watch = False
    files = []

    for a in args:
        if a == '--live':
            live = True
        elif a == '--watch':
            watch = True
        elif a.startswith('--'):
            print(f"Unknown option: {a}")
            sys.exit(1)
        else:
            files.append(a)

    if not files:
        print("Usage: python3 run_e.py [--live] [--watch] <file.e> ...")
        print("")
        print("  --live    Actually execute actions")
        print("  --watch   Keep process alive for scheduled tasks")
        sys.exit(1)

    ok = True

    for path in files:
        print(f"\n{'='*60}")
        print(f"▶️  E — {path}" + (" (LIVE)" if live else " (dry-run)"))
        print(f"{'='*60}")
        try:
            with open(path) as f:
                source = f.read()
            tokens = lex(source)
            parser = Parser(tokens)
            ast = parser.parse()
            executor = Executor(live=live)
            executor.run(ast)
            print(f"\n✅ Done: {path}")
        except (SyntaxError, ParseError) as e:
            print(f"❌ SYNTAX ERROR: {e}")
            ok = False
        except EError as e:
            print(f"❌ RUNTIME ERROR: {e}")
            ok = False
        except FileNotFoundError:
            print(f"❌ File not found: {path}")
            ok = False
        except KeyboardInterrupt:
            print("\n🛑 Interrupted")
            ok = False
            break

    if watch and live:
        print(f"\n⏳ Watching for scheduled tasks... Press Ctrl+C to stop.")
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            print("\n🛑 Stopped")

    if not ok:
        sys.exit(1)


if __name__ == '__main__':
    main()
