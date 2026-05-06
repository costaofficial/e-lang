# E — a general-purpose language

**E describes *when* to do something, *on what*, and *what to do*** — the runtime handles the rest.

## Quick start

```bash
git clone https://github.com/costaofficial/e-lang.git
cd e-lang/e
cargo run -- ../examples/hello.eee

# build the binary
cargo build --release
./target/release/e examples/hello.eee
```

## Install globally

```bash
cd e-lang/e
cargo build --release
sudo cp target/release/e /usr/local/bin/
e examples/hello.eee
```

## Example

```eee
:sys
use "libhttp.so"
use "libdb.so"

:core
fn pagina_utente id do
    let dati = sys_call "libdb.so" "query" "SELECT * FROM users WHERE id = " + id
    log dati
done

:ui
<script>
function mostra() { document.title = "E app"; }
</script>
```

## Examples

| File | What it does |
|------|-------------|
| `examples/g.e` | Garmin Connect: login, export, download, email |
| `examples/hello.e` | Opens Google, searches |
| `examples/backup.e` | Writes a file, emails it |
| `examples/bash_demo.e` | Bash replacement demo |
| `examples/when_demo.e` | Conditions demo |

## Documentation

- [GRAMMATICA.md](GRAMMATICA.md) — formal grammar (EBNF)
- [SPEC.md](SPEC.md) — full language specification

## Status

**v3.1** — 3-tier language: Rust plugins + E core + JS UI.
Single binary, no dependencies. Plugin system via `.so` modules.

## License

Apache 2.0
