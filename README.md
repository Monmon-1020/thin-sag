# 🚪 thin‑sag (Secure Agent Gateway) — **v0.3.0‑beta**

> **TL;DR**
> • macOS 14 only (Apple Silicon & Intel)
> • A drop‑in **Trust Layer** that lets any AI agent control your GUI safely
> • All‑in‑one binary: Vault isolation ▸ Policy engine ▸ Job queue ▸ Audit logs

---

## ✨ Key Features

| Module                   | What it does                                                                           |
| ------------------------ | -------------------------------------------------------------------------------------- |
| **JSON Action DSL**      | Declarative `launch / type / click / scroll / wait / keypress` – easy for LLMs         |
| **Vault Isolation**      | Secrets are pulled from macOS Keychain & never shown to the LLM                        |
| **Policy v0**            | YAML allow/deny for acts & targets ＋ click bounds ＋ wait‑limit                         |
| **Job Queue & REST API** | `POST /run‑json`  →  `GET /job/{id}`                                                   |
| **Masked UI Snapshot**   | `POST /snapshot` returns an Accessibility Tree with secrets auto‑masked (`***MASK***`) |
| **Audit Logs**           | Every action (pass / blocked) streamed to `~/.thin-sag/logs/*.jsonl`                   |

---

## ⏬ Installation

1. **Download** the notarized DMG → [thin-sag‑v0.3.0‑beta.dmg](https://github.com/your-org/thin-sag/releases/latest)
2. **Drag & Drop** `thin-sag.app` into **/Applications**
3. At first launch macOS asks for **Accessibility** permissions – click **Allow**
4. *(optional)* Symlink the binary into your `$PATH`:

```bash
ln -s /Applications/thin-sag.app/Contents/MacOS/thin-sag \
      /usr/local/bin/thin-sag
```

---

## 🚀 Quick Start

### 1 · Add a secret to the Vault

```bash
thin-sag vault add profile_name "Your Name"
```

### 2 · Start the local server

```bash
thin-sag serve --port 8900           # generates ~/.thin-sag/.sagtoken
```

### 3 · Run a **JSON Action** job

```bash
curl -H "X-SAG-TOKEN: $(cat ~/.thin-sag/.sagtoken)" \
     -H "Content-Type: application/json" \
     -d '[
           {"act":"launch",   "target":"com.apple.Notes"},
           {"act":"wait",     "ms":800},
           {"act":"type",     "text":"Hello {secret.profile_name}!"},
           {"act":"keypress", "key":"CMD+S"}
         ]' \
     http://127.0.0.1:8900/run-json
```

### 4 · Track job status

```bash
curl http://127.0.0.1:8900/job/<job_id> | jq .
```

### 5 · Take a masked UI snapshot

```bash
curl -H "X-SAG-TOKEN: $(cat ~/.thin-sag/.sagtoken)" \
     -H "Content-Type: application/json" \
     -d '{"window":"front"}' \
     http://127.0.0.1:8900/snapshot | jq .
```

*(secret values are replaced with `***MASK***`)*

---

## 📡 API Reference (Routers & Args)

| Route              | Method | Body / Params                                                                        | Purpose                      |
| ------------------ | ------ | ------------------------------------------------------------------------------------ | ---------------------------- |
| `/run`             | `POST` | `{"bundle":"<BundleID>","secret":"<label>","text":"Hello {secret.label}"}`           | legacy one‑shot login helper |
| `/run-json`        | `POST` | `Action[]` (see DSL below)                                                           | queue a multi‑step job       |
| `/job/{id}`        | `GET`  | –                                                                                    | returns `{status,result}`    |
| `/snapshot`        | `POST` | `{ "window":"front" \| {"window":{"index":N \| "title":"regex" \| "doc":"regex"}} }` | masked Accessibility Tree    |
| `/ui/log` *(beta)* | `GET`  | –                                                                                    | HTML list of audit logs      |

### JSON Action DSL v0.1

```jsonc
[
  { "act":"launch",   "target":"com.apple.Notes" },
  { "act":"click",    "x":200, "y":300 },
  { "act":"scroll",   "dy":-500 },
  { "act":"type",     "text":"{secret.email}" },
  { "act":"keypress", "key":"CMD+S" },
  { "act":"wait",     "ms":1000 }
]
```

---

## 🔧 Policy (YAML v0)

```yaml
allow_snapshot: true            # disable = block /snapshot
allow_acts: [launch,type,click,scroll,wait,keypress]
denied_targets:
  - "*.phishing.com"
  - "com.malware.*"
max_wait_ms: 30000
click_bounds:
  x_min: 0
  x_max: 2560
  y_min: 0
  y_max: 1600
```

*Edit & save → the new rules apply instantly.*

---

## 🛡️ Security Highlights

1. **Vault isolation** – the LLM never sees plaintext secrets
2. **Token auth** – every request needs `X-SAG-TOKEN`
3. **Policy gate** – invalid actions are blocked pre‑queue
4. **Audit trail** – immutable JSON Lines for all pass/fail
5. **Auto‑mask** – injected secrets & PII regexes → `***MASK***`

---

## 📈 Roadmap (Public)

| Milestone  | Planned Items                                     |
| ---------- | ------------------------------------------------- |
| **v0.3.1** | snapshot depth+window‑enum fix · CLI self‑update  |
| **v0.4.0** | Windows UIA adapter · Chrome WebExtension adapter |
| **v0.4.x** | GUI policy editor · live dashboard                |
| **v1.0**   | Masked PNG screenshot · plugin marketplace        |

---

## Known Limitations (beta)

* Snapshot covers **frontmost window only**, depth = 3
* Windows/Linux not yet supported
* `click.selector` is a stub – use `(x,y)` for now

---

## License

MIT © 2025 Secure Agent Gateway Project

snapshot
curl -H "X-SAG-TOKEN: $(cat ~/Desktop/thin-sag/.thin-sag/.sagtoken)" \
     -H "Content-Type: application/json" \
     http://127.0.0.1:8900/snapshot -d
