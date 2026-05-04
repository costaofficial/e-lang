# E — an automation language

**E describes *when* to do something, *on what*, and *what to do*** — the runtime handles the rest.

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

v0.1 — proof of concept. Parser + interpreter working in dry-run mode.
Browser automation, email, and file watching are stubs ready for real drivers.

## License

Apache 2.0
