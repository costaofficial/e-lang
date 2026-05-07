# E — Design document (original)

> v5.0 — May 2026

## What is E

E is a general-purpose language that describes **when** to do something, **on what**, and **what to do**. The runtime handles all complexity.

## Core syntax

```ebnf
program    = { statement_unit } ;
statement_unit = time_unit | script_unit | fn_definition | let_statement | use_statement ;
```

`... do ... done` for everything. No braces, no semicolons.

## 3-tier architecture (.eee files)

- `:sys` — plugin and module declarations
- `:core` — E code (logic, functions, expressions)
- `:ui` — HTML + JavaScript (native WebView window)

## Current capabilities

| Area | Status |
|------|--------|
| Variables, functions, expressions | ✅ |
| Browser automation (open, click, find, login, download) | ✅ headless_chrome |
| WebView (:ui) | ✅ wry + tao |
| Scheduling (time, --watch) | ✅ |
| File I/O, shell commands | ✅ |
| JSON, HTTP plugins | ✅ built-in |
| File-based JSON database | ✅ |
| Modules (use) | ✅ |
| Retry, fallback (or) | ✅ |
| String/list methods | ✅ split, join, sort, replace, ... |
| CLI arguments | ✅ args variable |

## Implementation

- **Written in:** Rust
- **Parser:** Hand-written recursive descent
- **Runtime:** AST interpreter with scope stack
- **Binary:** Single ~8MB file, no dependencies
- **License:** Apache 2.0
