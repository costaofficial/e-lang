# E — a general-purpose language

**E describes *when* to do something, *on what*, and *what to do*** — the runtime handles the rest.

## Quick start

```bash
git clone https://github.com/costaofficial/e-lang.git
cd e-lang/e
cargo run -- ../examples/hello.e

# build the binary
cargo build --release
./target/release/e examples/hello.e
```

## Install globally

```bash
cd e-lang/e
cargo build --release
sudo cp target/release/e /usr/local/bin/
e examples/hello.e
```

## Example

```rust
time every week at 03:00 do
    with browser do
        open "https://connect.garmin.com"
        with page { timeout: 15s } do
            login "user" "password" or stop
            click "#export-all-btn"
            wait download
        done
    done
    email to "admin@example.com" file "export.zip"
done or log error
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

**v2.0** — Rust runtime. Single binary, no dependencies.
Previously a Python prototype (v0.1–v1.2), fully rewritten in Rust.

## License

Apache 2.0
