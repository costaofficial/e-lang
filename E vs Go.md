# E vs Go — Comparison

> General-purpose languages: E (dynamic, Rust runtime) vs Go (static, native compiler)

---

## 1. Philosophy

| Aspect | E | Go |
|--------|---|----|
| Paradigm | Dynamic, declarative, event-oriented | Static, imperative, concurrent |
| Typing | Dynamic (unified f64, strings, lists) | Static, strong, compiled |
| Syntax | `when x > 5 do log "ok" done` | `if x > 5 { fmt.Println("ok") }` |
| Runtime | AST interpreter (Rust) | Native compiled binary |

---

## 2. Syntax comparison

### Hello World

**E:**
```eee
log "hello world"
```

**Go:**
```go
package main
import "fmt"
func main() {
    fmt.Println("hello world")
}
```

### Variables

**E:**
```eee
let x = 5
let name = "E"
let list = [1, 2, 3]
```

**Go:**
```go
var x = 5
name := "Go"
list := []int{1, 2, 3}
```

### Functions

**E:**
```eee
fn double n do n * 2 done
log double 21
```

**Go:**
```go
func double(n int) int {
    return n * 2
}
fmt.Println(double(21))
```

### Conditions

**E:**
```eee
when x > 5 and x < 10 do
    log "between"
done
```

**Go:**
```go
if x > 5 && x < 10 {
    fmt.Println("between")
}
```

### Loops

**E:**
```eee
for n in [1, 2, 3] do
    log n
done

let i = 0
while i < 5 do
    let i = i + 1
done
```

**Go:**
```go
for _, n := range []int{1, 2, 3} {
    fmt.Println(n)
}

for i := 0; i < 5; i++ {
    // ...
}
```

### Error handling

**E:**
```eee
click "#btn" or log "not found"

retry 3 times do
    login "user" "pass"
done or stop
```

**Go:**
```go
err := click("#btn")
if err != nil {
    log("not found")
}

for i := 0; i < 3; i++ {
    err := login("user", "pass")
    if err == nil {
        break
    }
}
```

---

## 3. Unique features

### E has that Go doesn't

| Feature | Example |
|---------|---------|
| Built-in browser automation | `with browser do open "url"; find "h1"; click; login "u" "p"; wait download done` |
| WebView / native UI | `:ui` section with HTML + JS in `.eee` files |
| Scheduling | `time every day at 02:00 do ... done` with `--watch` |
| File watching | `watch "downloads/" do ... done` |
| Retry with fallback | `retry 3 times do ... done or log "failed"` |
| Built-in HTTP client | `sys_call "http" "e_get" "https://api.example.com"` |
| Built-in JSON | `sys_call "json" "e_parse" data` |
| Built-in DB | `sys_call "db" "e_query" "users" "SELECT * FROM users"` |
| 3-tier files | `:sys` + `:core` + `:ui` in a single `.eee` file |
| Single binary, zero deps | Copy `e` anywhere, it runs |
| Dynamic typing | Variables change type freely |
| String/list methods | `"hello".replace "l" "x"`, `[3,1,2].sort` |
| Args from CLI | `args[0]` = script path, `args[1..]` = arguments |

### Go has that E doesn't

| Feature | What it enables |
|---------|----------------|
| Static typing | Compile-time error checking |
| Go routines + channels | Massive concurrency, CSP model |
| Native compilation | Blazing fast execution |
| Package ecosystem | `go get`, thousands of libraries |
| HTTP server (stdlib) | Built-in `net/http` |
| Generics (Go 1.18+) | Type-safe data structures |
| Interfaces | Polymorphism without inheritance |
| Cross-compilation | Build for any OS/arch |
| Memory safety | No null pointer exceptions |
| Garbage collector | Automatic memory management |
| Tooling | `gofmt`, `go vet`, `go test`, `pprof` |
| Deployment | Single binary (like E) |
| Maturity | 12+ years, Google-backed |

---

## 4. Performance

| Test | E (interpreter) | Go (compiled) |
|------|----------------|---------------|
| Startup time | ~0.05s (fast) | ~0.001s (instant) |
| CPU loop 1M | ~0.06s | ~0.003s |
| Binary size | ~8 MB (release) | ~2 MB |
| Memory (idle) | ~5-10 MB | ~0.5 MB |

Go is faster (compiled native code vs AST interpreter). But for scripting and automation, E's startup time (0.05s) is fast enough for any practical use.

---

## 5. Use cases

| Scenario | E | Go |
|----------|---|----|
| Scripting / automation | ✅ Native | ⚠️ Possible but verbose |
| Browser automation | ✅ Built-in | ❌ Needs Selenium/playwright |
| Web server | ⚠️ Via plugin | ✅ `net/http` stdlib |
| CLI tools | ✅ Single binary | ✅ Single binary |
| Microservices | ❌ No HTTP server yet | ✅ Excellent |
| Data processing | ⚠️ Possible | ✅ Fast + concurrency |
| IoT / embedded | ⚠️ Needs compilation | ✅ Tiny binary |
| Desktop UI | ✅ WebView | ❌ No native GUI |
| System programming | ❌ No syscalls | ✅ File descriptors, signals |
| Learning to code | ✅ Human-readable | ❌ Verbose for beginners |

---

## 6. Ecosystem comparison

| Metric | E | Go |
|--------|---|----|
| Age | 1 month | 12+ years |
| Creator | You | Google (Ken Thompson, Rob Pike) |
| Community | You | 1M+ developers |
| Libraries | 4 built-in (json, fs, db, http) | Thousands on pkg.go.dev |
| Package manager | Coming (`.epm`) | `go mod` |
| IDE support | VSCode (manual) | VSCode, GoLand, vim-go |
| Documentation | GRAMMAR.md + SPEC.md | Extensive official docs |

---

## Summary

| E wins when | Go wins when |
|-------------|-------------|
| You need browser automation | You need maximum performance |
| You want scripting with retry/scheduling | You need concurrent microservices |
| You want a single file to do everything (3-tier) | You need full type safety |
| You're learning or prototyping | You're building production systems |
| You want built-in plugins (JSON, HTTP, DB) | You need Go's ecosystem |

E is not a competitor to Go. E is a scripting/automation language with built-in browser, plugins, and UI. Go is a systems language for concurrent, high-performance services. They complement each other — you could write a microservice in Go and orchestrate it with E.
