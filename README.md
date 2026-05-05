# E — a general-purpose language

**E describes *when* to do something, *on what*, and *what to do*** — the runtime handles the rest.

## Example

```rust
time every week at 03:00 do
    with browser do
        open "https://connect.g.com"
        with page { timeout: 15s } do
            login "user" "password" or stop
            open "https://connect.g.com/modern/activities"
            click "#export-all-btn"
            wait download
        done
    done
    email to "admin@example.com" file "export.zip"
done or log error
```

## Quick start

```bash
# install globally
pip install git+https://github.com/costaofficial/e-lang.git

# dry-run (shows what would happen)
e examples/hello.e

# live execution
e --live examples/g.e

# keep alive for scheduled tasks
e --live --watch examples/g.e
```

## Examples

| File | What it does |
|------|-------------|
| `examples/g.e` | Garmin Connect: login, export 30 activities, download, email |
| `examples/hello.e` | Opens Google, searches |
| `examples/backup.e` | Writes a file, emails it |
| `examples/login.e` | Retry logic with fallback |
| `examples/download.e` | Browser pipeline |
| `examples/connect.e` | Generic browser automation |
| `examples/browser_demo.e` | Live browser demo |
| `examples/demo_live.e` | Live file + subprocess |
| `examples/bash_demo.e` | Bash replacement demo |
| `examples/when_demo.e` | Conditions demo |

## License

Apache 2.0
