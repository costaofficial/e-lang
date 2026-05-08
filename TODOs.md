<!-- Copyright (c) 2026 Costa -->
# E — Roadmap & TODOs

> v5.0 — May 2026

## Stato attuale

Tutte le feature core sono **complete**. E oggi è un linguaggio general-purpose funzionante con:

- ✅ Parser + runtime + AST completi
- ✅ Browser automation (headless_chrome)
- ✅ WebView (:ui con wry/tao)
- ✅ Scheduling (--watch con time blocks)
- ✅ Built-in plugins (json, fs, db, http)
- ✅ String/list methods (split, join, sort, replace, ...)
- ✅ CLI args (args variable)
- ✅ Init system (time every X, watch, retry)
- ✅ Single binary, zero dipendenze

## Prossimi passi possibili

| Priorità | Cosa | Tempo | Descrizione |
|----------|------|-------|-------------|
| 🟡 | **Compilatore LLVM** | 1-2 settimane | E compila a codice nativo |
| 🟡 | **EOS** | 2 settimane | Ubuntu minimal + E + WebView desktop |
| 🟡 | **Package manager (.epm)** | 1 settimana | `e install`, `e search` |
| 🟢 | **Linter + formattatore** | 2 giorni | `e check`, `e format` |
| 🟢 | **Testare E con progetti reali** | 1-2 giorni | Costruire qualcosa di concreto |

## Bug noti

- `wait_download` timeout di 30s se nessun file viene scaricato (non è un bug, è il comportamento atteso)

## Idee future

- Self-hosting (parser di E scritto in E)
- Redox OS port
- REJ (Rust + E + JavaScript) come stack applicativo completo
- Change licenza
