# E — general-purpose language

**E describes *when* to do something, *on what*, and *what to do*** — the runtime handles the rest.

## Quick start

```bash
# Download the latest binary
curl -L https://github.com/costaofficial/e-lang/releases/latest/download/e -o e
chmod +x ./e

# Run an example
./e examples/hello.eee
```

## Features

- **Scripting** — shell commands, file I/O, variables, functions, loops, conditions
- **Browser automation** — open, find, click, login, download via headless Chrome
- **Scheduling** — `time every day at 02:00 do` with `--watch` flag
- **WebView UI** — `:ui` section renders HTML/JS in native window
- **Built-in plugins** — json, fs, db, http as first-class modules
- **Single binary** — zero dependencies, no runtime required

## Example

```eee
:sys
use "http"
use "json"

:core
do
    let data = sys_call "http" "e_get" "https://api.example.com"
    let parsed = sys_call "json" "e_parse" data
    log parsed
done

:ui
<h1>E app</h1>
<script>fetch('/api').then(r => r.json());</script>
```

## CLI

```bash
./e script.eee                   # dry-run
./e --live script.eee            # live execution
./e --watch --live script.eee    # keep alive for scheduled tasks
./e script.eee arg1 arg2         # pass arguments (args variable)
```

## Examples

| File | What it does |
|------|-------------|
| `examples/hello.eee` | Browser: open, find, click, log |
| `examples/core.eee` | Language core tests |
| `examples/plugins.eee` | Standard library (json, fs, db, http) |
| `examples/login.eee` | Retry with fallback |
| `examples/when_demo.eee` | Conditions and `and` logic |
| `examples/g.eee` | Full 3-tier with scheduling |
| `examples/lib.eee` | Modules via `use` |

## Documentation

- [GRAMMAR.md](GRAMMAR.md) — formal grammar (EBNF)
- [SPEC.md](SPEC.md) — full language specification

## Download

Get the latest binary from the [Releases](https://github.com/costaofficial/e-lang/releases) page.

```bash
curl -L https://github.com/costaofficial/e-lang/releases/latest/download/e -o e
chmod +x ./e
./e examples/hello.eee
```

## License

Apache 2.0
