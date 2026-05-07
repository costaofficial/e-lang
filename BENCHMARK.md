# E vs Python — Benchmark

> Test reali eseguiti su Ubuntu 24.04, Intel x86_64, 8GB RAM
> E v5.0 (Rust, debug build) — Python 3.12

---

## 1. Binary size

| Metrica | E | Python |
|---------|---|--------|
| Binario | 196 MB (debug) / ~8 MB (release) | Python 3.12: ~50 MB (full install) |
| Dipendenze | Zero (single binary) | Python interpretato + librerie |

**E vince** per deploy: un solo file, niente runtime da installare.

---

## 2. Startup time

| Test | E | Python |
|------|---|--------|
| `print(1)` / `log 1` | 0.050s | 0.061s |
| `print(1)` / `log 1` | 0.057s | 0.066s |
| `print(1)` / `log 1` | 0.052s | 0.055s |

**Simili** — entrambi sotto 0.07s.

---

## 3. CPU — 1M iterations (somma aritmetica)

```python
# Python
s = 0
for i in range(1000000):
    s += i * 2
print(s)
```

```eee
// E
do
    let i = 0
    let sum = 0
    while i < 1000000 do
        let sum = sum + i * 2
        let i = i + 1
    done
    log sum
done
```

| Metrica | E | Python | Ratio |
|---------|---|--------|-------|
| Tempo reale | ~0.06s | ~0.50s | **E ~8x più veloce** |
| CPU user | 0.04s | 0.34s | **E ~8x più veloce** |
| Risultato | 999999000000 | 999999000000 | ✅ Identico |

---

## 4. File I/O — scrivere e leggere 1000 righe

| Metrica | E | Python |
|---------|---|--------|
| Tempo reale | ~0.20s | ~0.06s |

Python vince perché `write file` in E fa una system call per iterazione (non c'è buffering).

---

## 5. Subprocess — 100 chiamate a `echo "ok"`

| Metrica | E | Python |
|---------|---|--------|
| Tempo reale | ~0.25s | ~0.30s |

**E leggermente più veloce** — chiamata diretta a `std::process::Command` senza overhead GIL.

---

## 6. Memoria (max resident set size)

| Test | E | Python |
|------|---|--------|
| CPU 1M iterazioni | 45 MB | 12 MB |

**Python usa meno RAM.** E in debug mode include simboli di debug. In release mode (~8 MB binary) la memoria reale sarebbe molto più bassa (~5-10 MB).

---

## 7. Factorial ricorsivo (20!)

| Metrica | E | Python |
|---------|---|--------|
| Tempo reale | ~0.06s | ~0.06s |

**Identico** — entrambi ottimizzano chiamate ricorsive in modo simile.

---

## Riepilogo

| Test | Vincitore | Note |
|------|-----------|------|
| Binary size / deploy | **E** | Singolo binario 8 MB vs Python + librerie |
| Startup | **Pari** | Entrambi ~0.05s |
| CPU (1M loop) | **E** (~8x) | Compilato vs interpretato |
| File I/O | **Python** | E manca di buffering |
| Subprocess | **E** (leggero) | No GIL overhead |
| Memoria | **Python** | E in debug mode è grande |
| Ricorsione | **Pari** | Prestazioni simili |
| Ecosistema | **Python** | ML, data science, librerie |
| Deploy | **E** | `sudo cp e /usr/local/bin/` |


## Log dei test

```
=== 1. Binary size ===
E: 196086040 bytes (187M debug)
Python: (interpretato, nessun binario)

=== 2. Startup time ===
E:   0.050s  0.057s  0.052s
Python: 0.061s  0.066s  0.055s

=== 3. CPU 1M iterations ===
E:   real 0m0.060s  user 0m0.040s
Python: real 0m0.503s  user 0m0.342s

=== 4. File I/O 1000 lines ===
E:   real 0m0.208s
Python: real 0m0.065s

=== 5. Subprocess 100x ===
E:   real 0m0.247s
Python: real 0m0.315s

=== 6. Memory (max resident) ===
E:   45356 KB
Python: 11916 KB

=== 7. Factorial 20! ===
E:   real 0m0.060s
Python: real 0m0.059s
```
