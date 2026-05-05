# E — a general-purpose language

**E describes *when* to do something, *on what*, and *what to do*** — the runtime handles the rest.
Scripts. Backend. Automation. Whatever you need.

## Example

```rust
time every day at 02:00 do
    with browser do
        open "https://connect.garmin.com"
        with page { timeout: 10s } do
            login "user" "pass" or stop
            click "#export-all"
            wait download
        done
    done
done or log error
```

## Quick start

```bash
# dry-run (shows what would happen)
python3 runtime/run_e.py examples/hello.e

# live mode (actually runs actions)
python3 runtime/run_e.py --live examples/backup.e
```

## Examples

| File | What it does |
|------|-------------|
| `examples/hello.e` | Opens Google, searches |
| `examples/backup.e` | Writes a file, emails it |
| `examples/login.e` | Retry logic with fallback |
| `examples/download.e` | Browser automation pipeline |
| `examples/garmin.e` | Full automation script |

## Grammar

See [GRAMMATICA.md](GRAMMATICA.md) for the full EBNF specification.

## Status

v0.2 — live execution enabled.

| Feature | Dry-run | Live |
|---------|---------|------|
| Parser + AST | ✅ | ✅ |
| `log` | ✅ | ✅ |
| `run` (subprocess) | ✅ | ✅ |
| `write` / `create` (filesystem) | ✅ | ✅ |
| `open` (browser) | ✅ | ✅ |
| `time` scheduling | ✅ | ✅ |
| `retry` + `fallback` | ✅ | ✅ |
| `click` / `find` | ✅ | ✅ (Playwright) |
| `login` | ✅ | ⏳ |
| `email` | ✅ | ⏳ |
| `wait download` | ✅ | ⏳ |
| `watch` (filesystem) | ✅ | ⏳ |

## Install

```bash
python3 runtime/run_e.py examples/hello.e          # dry-run
python3 runtime/run_e.py --live examples/garmin.e  # live

# optional: browser automation
pip install playwright
playwright install chromium
```

## Examples

| File | What it does |
|------|-------------|
| `examples/hello.e` | Opens Google, searches |
| `examples/backup.e` | Writes a file, emails it |
| `examples/login.e` | Retry logic with fallback |
| `examples/download.e` | Browser pipeline |
| `examples/garmin.e` | Full script |
| `examples/browser_demo.e` | Live browser demo |
| `examples/demo_live.e` | Live file + subprocess |

## License

Apache 2.0
