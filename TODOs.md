<!-- Copyright (c) 2026 Costa -->
# E — Roadmap & TODOs

> EOS v0.1 — May 2026 | E language v5.2.0

## ✅ Fatto (v5.0 → v5.2.0)

### E Language
- [x] Parser + runtime + AST completi
- [x] Browser automation (headless_chrome)
- [x] WebView (:ui con wry/tao)
- [x] Scheduling (--watch con time blocks)
- [x] Built-in plugins (json, fs, db, http)
- [x] String/list methods (split, join, sort, replace, ...)
- [x] CLI args (args variable)
- [x] Init system (time every X, watch, retry)
- [x] Single binary, zero dipendenze

### Compilatore LLVM (`e build`)
- [x] Float/int, stringhe, funzioni utente, dynamic llc
- [x] `for` (list unrolling + range loop)
- [x] `and`/`or` con short-circuit
- [x] Variabili stringa + strcmp + snprintf
- [x] Heap-allocated list + `for n in xs`
- [x] `len`, `not`, `name[0]`, `name[0..1]`
- [x] `run "cmd"` via `system()`
- [x] `--kernel` flag: bare metal x86_64
- [x] Built-in kernel: `vga_clear`, `vga_string`, `vga_raw_write`, `halt`
- [x] Interactive REPL (E Shell)
- [x] E Shell: PATH lookup, spawn, cd, clear
- [x] `--url` WebView browser
- [x] Multiboot2 header (GRUB 2.12+)
- [x] core.eee **9/9** — 0 warnings

### EOS Kernel (github.com/costaofficial/eos)
- [x] Kernel 100% E su x86_64 bare metal
- [x] Multiboot2 + GRUB ISO
- [x] VGA text mode (0xB8000)
- [x] QEMU + VNC + noVNC access via browser
- [x] Linker script + Makefile + GitHub repo

### Performance
- [x] core.eee: 9/9 tests passano nativamente
- [x] ~100x vs Python, ~2x vs Go in loop numerici
- [x] 16 MB single binary, zero runtime dependencies
- [x] 0 compiler warnings
- [x] 5 repo GitHub (e-lang, e-runtime, eos, e-shell, e-browser)

## 🔜 Piani: EOS — Sistema Operativo completo in puro E

### Fase 1: Input (PS/2 Keyboard + Port I/O) — ~2 settimane
- [ ] Built-in kernel: `inb`, `outb` (port I/O)
- [ ] PS/2 keyboard driver (polling mode)
- [ ] Scancode → ASCII conversion table
- [ ] Keyboard buffer + read key function
- [ ] Demo: echo tastiera su VGA

### Fase 2: Shell su framebuffer — ~2 settimane
- [ ] VGA character buffer scrolling
- [ ] E Shell compilata per kernel mode
- [ ] Comandi built-in (ls → VGA, cd → VGA)

### Fase 3: Memory + File System — ~4-6 settimane
- [ ] Heap allocator (malloc per kernel)
- [ ] Page table management
- [ ] FAT32 driver (read + write)
- [ ] File operations (open, read, write)

### Fase 4: Init System — ~1 settimana
- [ ] PID 1 in E
- [ ] Service management
- [ ] Boot automatico → E Shell

### Fase 5: GUI Desktop — ~3-4 settimane
- [ ] Framebuffer graphics (VESA)
- [ ] Window manager in E
- [ ] Icone + menu + launcher
- [ ] Terminal window integrato

### Fase 6: Rete — ~3-6 mesi
- [ ] e1000 driver
- [ ] TCP/IP stack
- [ ] HTTP client in kernel
- [ ] E App Store via rete

## Bug noti
- `vga_raw_write` in while loop → FIXATO
- `ld` avvisa RWE segment (innocuo per kernel)
- GRUB "address out of range" → workaround: ISO con multiboot2

## Record storici
- Primo kernel 100% E su x86_64 bare metal (May 2026)
- E è il 3° linguaggio nella storia (dopo Oberon e HolyC) il cui autore ha creato sia il linguaggio che un kernel funzionante
- Unico linguaggio con: compilatore LLVM + kernel bare metal + shell ibrida + browser WebView + REPL
