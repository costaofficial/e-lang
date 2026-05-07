# E — Roadmap & TODOs

> Ultimo aggiornamento: v4.3

---

## 📦 Priorità alta — completare il linguaggio

### 1. Browser automation reale (2-3 giorni)

Sostituire gli stub in `browser.rs` con chiamate reali a Chromium via `chromiumoxide` crate.

```
- open url → apre Chrome, naviga
- find selector → aspetta e trova elemento
- click → clicca elemento trovato
- login user pass → riempie form login e submit
- wait download → aspetta download e restituisce path
- wait until visible/hidden → aspetta condizione DOM
```

**Dipendenze:** `chromiumoxide = "0.7"`

**File:** `e/src/browser.rs`

---

### 2. WebView funzionante (:ui) (1 giorno)

Fixare `ui.rs` per visualizzare HTML nella finestra (oggi apre solo finestra vuota).

```
- Usare wry + tao per creare WebView con contenuto HTML
- Se :ui ha script JS, eseguirli
- Se :core ha una funzione esposta, :ui può chiamarla
```

**File:** `e/src/ui.rs`

---

### 3. Init system (`time every boot`, `--watch` stabile) (1 giorno)

```
- time every boot do → esegue all'avvio (senza orario specifico)
- time every X at Y do → tiene vivo il processo e programma
- --watch flag → mantiene alive per schedule/watch
- watch path do → polling ogni 2 secondi per nuovi file
```

**File:** `e/src/runtime.rs`

---

### 4. Metodi per stringhe (1 giorno)

```eee
"hello world".split " "   → ["hello", "world"]
"hello".contains "ell"    → true
"hello".replace "l" "x"   → "hexxo"
"  hi  ".trim             → "hi"
"HELLO".lower             → "hello"
"hello".upper             → "HELLO"
```

**AST:** `Expr::Method(self, name, args)`

**File:** `e/src/runtime.rs` (handler in `eval_expr`)

---

### 5. Metodi per liste (1 giorno)

```eee
[3, 1, 2].sort            → [1, 2, 3]
[1, 2, 3].filter fn x do x > 1 done  → [2, 3]
[1, 2, 3].map fn x do x * 2 done     → [2, 4, 6]
[1, 2, 3].join ","        → "1,2,3"
[1, 2, 3].get 1           → 2
[1, 2, 3].len             → 3
```

**File:** `e/src/runtime.rs`

---

## 🔧 Priorità media — tooling e qualità

### 6. Argomenti da riga di comando in E

```eee
log args       → ["script.eee", "arg1", "arg2"]
log args[0]    → "script.eee"
```

**File:** `parser/main.rs` + runtime

---

### 7. Package manager (.epm)

```bash
e install http    # scarica e installa plugin/script
e update          # aggiorna tutto
e search web      # cerca pacchetti
```

**Idea:** File `.epm` come manifest, repository su GitHub. Ogni package = file `.eee` o plugin `.eso`.

---

### 8. Linter + formattatore

```bash
e check script.eee    # analizza errori senza eseguire
e format script.eee   # formatta automaticamente
```

---

### 9. Completamento automatico per shell

```bash
e [TAB][TAB]
# mostra: --live, --watch, --version, script.eee, ...
```

---

## 🚀 Visione — EOS (dopo che E è completo)

### 10. EOS — Ubuntu minimal + E + WebView

| Fase | Cosa | Tempo |
|------|------|-------|
| 1 | Ubuntu minimal + E preinstallato | 1 giorno |
| 2 | Init in E (sostituisce bash/systemd) | 2-3 giorni |
| 3 | Desktop WebView (dock, app, browser E) | 3-5 giorni |
| 4 | Sicurezza AppArmor per E | 2-3 giorni |
| 5 | Toolkit EOS (ls, cat, cp, mv in E) | 2-3 giorni |
| **TOTALE** | | **~2 settimane** |

### 11. Migrazione graduale verso Rust

| Layer | Ora | Futuro | Quando |
|-------|-----|--------|--------|
| Init | systemd | E script | Fase 2 |
| Shell | bash | E | Fase 1 |
| Package manager | apt | E package | Fase 3 |
| Display server | X11 | Rust (wry/tao) | Fase 5 |
| Kernel | Linux | Redox | 1-2 anni |

---

## 🧠 Note di design

### Sicurezza (capability-based per plugin)

```eee
:sys
use "http"      // concede rete
use "fs"        // concede filesystem
use "db"        // concede database

:core
// SENZA "fs", non puoi fare read/write
// SENZA "http", non puoi fare richieste
```

Su Ubuntu: profilo AppArmor generato automaticamente da `:sys`.
Su Redox: capability native del kernel.

### E come shell

```bash
# /etc/passwd
costa:x:1000:1000:,,,:/home/costa:/usr/bin/e

# All'avvio: prompt interattivo E>
# I comandi non riconosciuti vengono eseguiti come run "comando"
```

---

## 📊 Stato attuale (v4.3)

| Area | Stato |
|------|-------|
| Parser + AST | ✅ |
| Runtime (expr, exec) | ✅ |
| Variabili, fn, condizioni | ✅ |
| Liste (base) | ✅ |
| Stringhe (base) | ✅ |
| Moduli (use) | ✅ |
| Browser automation | ❌ stub |
| WebView | ❌ finestra vuota |
| Init / scheduling | ⚠️ funziona ma non persistente |
| Metodi stringhe | ❌ solo len/index/slice |
| Metodi liste | ❌ solo append |
| Plugin json/fs/db/http | ✅ built-in |
| Installazione | ✅ cargo build + cp |
| Test | ✅ cargo test 2 test |
| Package manager | ❌ |
| Linter | ❌ |
| EOS | ❌ pianificato |
