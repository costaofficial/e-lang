<!-- Copyright (c) 2026 Costa -->
# E — Roadmap & TODOs

> v5.2.0 — May 2026

## ✅ Fatto (v5.0 → v5.2.0)

### Compilatore LLVM (`e build`)
- [x] Float/int support (mixed mode con `has_float` flag)
- [x] Stringhe (globali + printf/snprintf)
- [x] Funzioni utente (two-pass compilation, parametri, return)
- [x] Dynamic llc detection (llc-18 → ... → llc)
- [x] Unhandled nodes (for/watch/time/retry → commenti runtime)
- [x] `for i in [1,2,3]` — unrolling liste letterali
- [x] `for i in 0..10` — range loop con contatore nativo LLVM
- [x] `and`/`or` con short-circuit evaluation (alloca + branch)
- [x] Variabili stringa (type-track, alloca i8*, strcmp)
- [x] String concat mista via snprintf (256-byte buffer sicuro)
- [x] Heap-allocated list: `let xs = [1,2,3]`, `xs.append 4`, `xs[i]`
- [x] `for n in xs` — iterazione su lista variabile
- [x] `len "hello"` (compile-time) e `len name` (strlen/inline)
- [x] `not expr`
- [x] `name[0]`, `name[0..1]` — string indexing/slicing
- [x] `let i = i + 1` — riassegnazione variabili (while loops)
- [x] `run "cmd"` via `system()` extern
- [x] `--kernel` flag: bare metal x86_64 (multiboot, VGA, halt)
- [x] Built-in kernel functions: `vga_clear`, `vga_string`, `vga_raw_write`, `halt`
- [x] Interactive REPL (`e` senza argomenti)

### Performance
- [x] core.eee: **9/9 tests passano nativamente** (compilato)
- [x] ~100x vs Python, ~2x vs Go in loop numerici
- [x] 16 MB single binary, zero runtime dependencies
- [x] 0 compiler warnings

### EOS Kernel (github.com/costaofficial/eos)
- [x] Kernel 100% E su x86_64 bare metal
- [x] Multiboot1 compatibile (QEMU -kernel)
- [x] VGA text mode output
- [x] Linker script + Makefile + GitHub repo
- [x] `.gitattributes` per rilevamento linguaggio E

## 🔜 Prossimi passi

| Priorità | Cosa | Tempo | Descrizione |
|----------|------|-------|-------------|
| 🔴 | **E Shell** | 1 ora | `cd`, `ls`, `echo`, `clear`, PATH, env vars |
| 🔴 | **TODOs aggiornato** | 5 min | questo è fatto |
| 🟡 | **Package manager (.epm)** | 1 settimana | `e install`, `e search` |
| 🟡 | **EOS v0.2** | 1 settimana | GDT/IDT, keyboard, shell su QEMU |
| 🟢 | **Linter + formattatore** | 2 giorni | `e check`, `e format` |
| 🟢 | **Pulire binari di test** | 10 min | rimuovere core, test_* da e-runtime |

## Bug noti

- `vga_raw_write` non funziona dentro `while` loops (built-in non raggiunto in funzioni utente)
- `ld /tmp/eos/kernel.elf` avvisa RWE segment (innocuo per kernel)

## Idee future

- Self-hosting (parser di E scritto in E)
- Pipe `|` nella E Shell
- Redox OS port
- REJ (Rust + E + JavaScript)
