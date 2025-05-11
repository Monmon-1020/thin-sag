🚪 **thin-sag** (Secure Agent Gateway) — v0.3.0-beta  
_A minimal trust layer for safely running any AI agent’s GUI actions on macOS 14 (Intel & Apple Silicon)._

---

## 🔑 Key Benefits

- **Vault isolation**  
  Secrets live only in your Keychain, injected on-demand and never exposed to agents.

- **Pre-action policy engine**  
  YAML rules to allow/deny actions & targets, enforce click bounds and wait limits.

- **Cross-OS UI adapters**  
  (macOS AX today; Windows UIA coming) share the same JSON/Wasm-call interface.

- **Immutable audit trail**  
  Every action → JSONL log; easy SIEM/ SOC integration.

- **Masked UI snapshot**  
  `/snapshot` returns an AX tree (and soon PNG) with secrets & PII auto-masked.

---

## ⏬ Installation

1. **Download the ZIP** from GitHub Releases:  
   ```bash
   curl -Lo thin-sag.zip \
     https://github.com/your-org/thin-sag/releases/download/v0.3.0-beta1/thin-sag-macos-v0.3.0-beta1.zip
Unzip & Install

```bash
unzip thin-sag.zip
Bypass Gatekeeper quarantine
```

```bash
xattr -dr com.apple.quarantine thin-sag
chmod +x thin-sag
```
Grant Accessibility when prompted on first run.

🚀 Quick Start
Add a secret

```bash
thin-sag vault add profile_name "Your Secret Value"
```
Start the server

```bash
thin-sag serve
```
Track job status

```bash
curl -H "X-SAG-TOKEN: $(cat ~/.thin-sag/.sagtoken)" \
     http://127.0.0.1:8900/job/<job_id> | jq .
```
List windows

```bash
curl http://127.0.0.1:8900/windows | jq .
```
Take a masked UI snapshot

```bash
curl -H "X-SAG-TOKEN: $(cat ~/.thin-sag/.sagtoken)" \
     -H "Content-Type: application/json" \
     -d '{"window":{"index":1}}' \
     http://127.0.0.1:8900/snapshot | jq .
```
Take a screenshot

```bash
curl -H "X-SAG-TOKEN: $(cat ~/.thin-sag/.sagtoken)" \
     http://127.0.0.1:8900/screenshot --output screen.png
```
📡 API Reference
All endpoints except /windows require the X-SAG-TOKEN header.

Route	Method	Body / Params	Description
/run	POST	{ "bundle": "...", "secret":"...", "text":"..." }	Legacy one-shot login helper
/run-json	POST	Action[]	Queue a multi-step job
/job/{id}	GET	–	Check job status & result
/windows	GET	–	List available windows (index & title)
/snapshot	POST	{ "window": { "index":N | "title":"regex" | "doc":"regex" } }	Masked Accessibility tree
/screenshot	GET	–	Return desktop screenshot (PNG)
/ui/log	GET	–	(beta) HTML list of audit logs

🔧 JSON Action DSL v0.1
```json
[
  { "act":"launch",   "target":"com.apple.Notes" },
  { "act":"click",    "x":200,  "y":300 },
  { "act":"scroll",   "dy":-500 },
  { "act":"type",     "text":"{secret.email}" },
  { "act":"keypress", "key":"CMD+S" },
  { "act":"wait",     "ms":1000 }
]
```
🛡️ Policy (YAML v0)
``yaml
allow_snapshot: true            # false to disable /snapshot
allow_acts:     [launch,type,click,scroll,wait,keypress]
denied_targets:
  - "*.phishing.com"
  - "com.malware.*"
max_wait_ms:    30000
click_bounds:
  x_min: 0
  x_max: 2560
  y_min: 0
  y_max: 1600
```
Edit & save → rules apply instantly.

📈 Roadmap (Public)
Version	Planned Features
v0.3.1	Snapshot depth & window enumeration fixes · CLI self-update
v0.4.0	Windows UIA adapter · Chrome WebExtension adapter
v0.4.x	GUI policy editor · live dashboard
v1.0	Masked PNG screenshot · plugin marketplace

❗ Known Limitations (beta)
Snapshot: frontmost window only, depth ≤ 3

Windows & Linux adapters not yet implemented

click.selector is a stub—use (x,y) for now

PNG masking not yet available; use policy to disable /screenshot if needed

📄 License
MIT © 2025 Secure Agent Gateway Project