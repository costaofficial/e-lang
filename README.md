# E — general-purpose language

**E describes *when* to do something, *on what*, and *what to do*** — the runtime handles the rest.

## Quick start

```bash
git clone https://github.com/costaofficial/e-lang.git
cd e-lang/e
cargo build --release
sudo cp target/release/e /usr/local/bin/

# run
e examples/hello.eee
e --live examples/hello.eee
e --watch --live examples/g.eee
```

## Features

- **Scripting** — shell commands, file I/O, variables, functions, loops, conditions
- **Browser automation** — open, find, click, login, download via headless Chrome
- **Scheduling** — `time every day at 02:00 do` with `--watch` flag
- **WebView UI** — `:ui` section renders HTML/JS in native window
- **Built-in plugins** — json, fs, db, http as first-class modules
- **Plugin system** — Rust `.so` modules callable from E
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
e script.eee                   # dry-run (show what would happen)
e --live script.eee            # live execution
e --watch --live script.eee   # keep alive for scheduled tasks
e script.eee arg1 arg2         # pass arguments (available as args variable)
```

## Examples

| File | What it does |
|------|-------------|
| `examples/hello.eee` | Browser: open, find, click, log |
| `examples/bash_demo.eee` | Shell commands, variables, conditions |
| `examples/core.eee` | Language core tests |
| `examples/plugins.eee` | Standard library (json, fs, db, http) |
| `examples/login.eee` | Retry with fallback |
| `examples/when_demo.eee` | Conditions and `and` logic |
| `examples/g.eee` | Full 3-tier with scheduling |
| `examples/lib.eee` | Modules via `use` |

## Documentation

- [GRAMMATICA.md](GRAMMATICA.md) — formal grammar (EBNF)
- [SPEC.md](SPEC.md) — full language specification
- [TODOs.md](TODOs.md) — roadmap and pending items

## Status

**v5.0** — Rust binary. Complete browser, WebView, plugins, scheduling.
Single binary, no dependencies.

## License

Apache 2.0
