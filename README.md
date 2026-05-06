# E — a general-purpose language

**E describes *when* to do something, *on what*, and *what to do*** — the runtime handles the rest.

## Features

- **Scripting** — shell commands, file I/O, variables, functions, conditions
- **Automation** — browser, email, scheduling, file watching, retry
- **Plugin system** — load Rust `.so` modules and call them from E
- **3-tier files** — `:sys` (Rust plugins) + `:core` (E logic) + `:ui` (HTML/JS) in one `.eee` file
- **Zero dependencies** — single binary, no runtime required

## Quick start

```bash
# install
git clone https://github.com/costaofficial/e-lang.git
cd e-lang/e && cargo build --release
sudo cp target/release/e /usr/local/bin/

# run
e examples/hello.eee
e --live examples/bash_demo.eee
```

## Example

```eee
:sys
use "db.eso"
use "http.eso"

:core
fn salva_utente nome do
    sys_call "db.eso" "insert" nome
    log "utente salvato: " + nome
done

let users = sys_call "db.eso" "query" "SELECT * FROM users"
for u in users do log u done

:ui
<h1>App</h1>
<script>alert('pronto');</script>
```

## Examples

| File | What it does |
|------|-------------|
| `examples/hello.eee` | Browser: open, find, click, log |
| `examples/backup.eee` | File write + email |
| `examples/bash_demo.eee` | Shell commands, variables, conditions |
| `examples/when_demo.eee` | Conditions and logic |
| `examples/login.eee` | Retry with fallback |
| `examples/g.eee` | Full 3-tier script with plugin |
| `examples/lib.eee` | Modules (use) |

## Documentation

- [GRAMMATICA.md](GRAMMATICA.md) — formal grammar (EBNF)
- [SPEC.md](SPEC.md) — full language specification

## License

Apache 2.0
