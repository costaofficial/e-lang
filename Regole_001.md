# 🧠 WHAT IS E (in one sentence)

> E is a general-purpose language that describes **when to do something, on what, and what to do**, leaving all complexity to the runtime.

---

# 🧱 1) THE BUILDING BLOCKS

## ⏰ Time (when)

```rust
time every hour at 00 do
```

👉 defines **when everything starts**

---

## 🧱 Objects (on what)

```rust
file "01.md"
app "Chrome"
browser
page
```

👉 these are the "things" in the world

---

## ⚙️ Actions (what you do)

```rust
create
write
open
click
upload
email
```

👉 these are the verbs

---

## 🧩 Structure (how it runs)

```rust
with ... do
done
or
```

👉 controls the flow

---

# 🔥 2) FULL STRUCTURE

```rust
time ... do
    actions
done or fallback
```

👉 always this pattern

---

# 🧠 3) CONTEXT (`with`)

```rust
with file "01.md" do
    write file "01.md" "Hello"
done
```

👉 means:

> work on this object

⚠️ rules you decided:

* must already exist ✔️
* does not auto-create ✔️

---

# 🎯 4) DIRECTION (`to`)

```rust
email to "info@..." file "01.md"
```

👉 pattern:

```text
action → to → destination → object
```

---

# ⚠️ 5) ERRORS

## 🔴 Global

```rust
done or log error
```

👉 any error → here

---

## 🟡 Local

```rust
login "user" "pass" or do
    log "failed"
    stop
done
```

👉 specific error

---

# 🔁 6) RETRY

```rust
retry 3 times do
    click "#btn"
done
```

👉 automatically retries

---

# ⏱️ 7) WAITING

## ✅ Auto-wait (default)

```rust
click "#login"
```

👉 waits automatically for:

* element visible
* element clickable

---

## 🔧 Manual

```rust
wait until visible "#chart"
wait until hidden ".loading"
```

---

# 📦 8) DOWNLOAD (hard case)

```rust
wait download
```

👉 under the hood:

1. browser event
2. filesystem
3. polling

---

# 🌐 9) WEB AUTOMATION

```rust
with browser do
    open "site"

    with page do
        find "login"
        click
    done
done
```

👉 layers:

* browser → window
* page → content
* element → target

---

# 🔐 10) LOGIN

```rust
login "user" "pass"
```

👉 two modes:

* auto (smart detect)
* manual (selector)

---

# ⏳ 11) TIMEOUT

```rust
with page { timeout: 10s } do
```

👉 prevents infinite blocking

---

# 🔄 12) SCRIPT vs AUTOMATION

## Script (immediate)

```rust
do
    run "ls"
done
```

---

## Automation

```rust
time every day at 08:00 do
```

---

# ⚙️ 13) HOW IT REALLY WORKS (under the hood)

```text
E code
↓
Parser
↓
E2 (structure)
↓
Rust Runtime
↓
Operating system
```

---

# ⚡ 14) ASYNCHRONY

👉 for the user:

* sequential

👉 under the hood:

* async / event-driven

---

# 🧠 15) MENTAL MODEL

👉 you write:

> what you want + when

👉 the system handles:

* waits
* retries
* error handling
* performance

---

# 💥 16) FINAL COMPLETE EXAMPLE

```rust
time every day at 02:00 do
    with browser do
        open "https://connect.garmin.com"

        with page { timeout: 10s } do
            login "user" "password" or stop

            click "export"

            retry 3 times do
                wait download
            done
        done
    done

    watch "downloads/" do
        with file "*.fit" do
            upload to "fitness.db"
            log imported
        done
    done

done or log error
```

---

# 🎯 FINAL SUMMARY

E is:

* declarative ✔️
* readable ✔️
* event-oriented ✔️
* powerful runtime ✔️

---

# 💥 THE MOST IMPORTANT SENTENCE

> E is not code
> it's a description of actions in time

---

# 🚀 Next step

We can:

👉 **official language specification (real docs)**
or
👉 **start writing the parser in Rust**

Where do you want to go? 😄
