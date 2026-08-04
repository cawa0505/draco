# Draco (fork)

> **Draco (fork)**: "Born out of frustration with bloated browser snapshots. An ultra-dehydrated Rust MCP server with zero browser-boot overhead and compact agent-facing output."

A fast, stealth, **native-Rust web scraper** — a lighter alternative to Firecrawl and Browserbase. Point it at a URL and get clean **Markdown + metadata** back, using a browser-faithful TLS/JA4 fingerprint to reach pages that block ordinary clients. No Node, no headless-Chrome fleet, no per-request browser boot.

Current development version: **v0.24.0**, adding the completed Phase 2 interact session tools. The plugin framework and any unfinished pre-Phase 3 work are planned for **v0.25.0**.

This fork specifically focuses on **strengthening the MCP layer and ergonomics for AI-agent use** (see [spec/mcp-agent-ergonomics/spec.md](spec/mcp-agent-ergonomics/spec.md)).

---

## Quick Start

The fastest way to install Draco on Linux or macOS is via the install script:

```sh
# Prebuilt binary installation
curl -fsSL https://raw.githubusercontent.com/cawa0505/draco/main/install.sh | sh

# Or build from this repository's source
./install.sh --from-source
```

Then scrape a page directly:
```sh
draco scrape https://example.com          # → clean Markdown on stdout
```

---

## Built for AI Agents: The MCP Ergonomics Leap 🚀

While traditional scrapers output raw markdown or require heavy headless browsers (like Playwright) for interaction, Draco bridges the gap. It provides **browser-level agent-interaction capabilities without the browser-boot overhead**, operating directly in an in-process V8 sandboxed environment.

This fork specifically redesigns the MCP layer to solve the primary friction points of AI-driven automation:

### Choose by Workload

| Workload | Use |
| :--- | :--- |
| Static pages and direct APIs | Traditional scraper |
| DOM extraction and lightweight interaction | Draco |
| Screenshots and browser-only behavior | agent-browser |

### Draco vs [agent-browser](https://github.com/vercel-labs/agent-browser)

| Area | Draco | agent-browser |
| :--- | :--- | :--- |
| Runtime | In-process DOM + V8 | Chromium |
| Targeting | Self-healing a11y refs | Browser refs |
| Layout and screenshots | No | Yes |
| Rate limits | Jitter, retry, proxy rotation | Browser/proxy setup |
| Role | Primary scraper | Browser fallback |

Use Draco first when the task only needs page content, structured data, or DOM interaction. Escalate to agent-browser when the task depends on real layout, screenshots, downloads, or browser-only APIs.

### Measured Performance

Draco v0.23.0 release binary, Linux x86_64 on an AMD Ryzen 5 5600GT. Each flow used a fixed localhost HTML fixture, 2 warm-up calls, then 30 successful sequential samples with response-content validation. These numbers measure local runtime and transport overhead, not internet latency or protected-site success.

| Flow | Latency p50 / p95 | Sequential rate |
| :--- | :--- | ---: |
| MCP stdio scrape | 1.33 / 1.69 ms | 724 calls/s |
| MCP HTTP scrape | 3.79 / 5.02 ms | 267 calls/s |
| HTTP interact lifecycle | 106.87 / 112.81 ms | 9.32 flows/s |

The interact lifecycle is `open → snapshot → clickRef → scrape → close`, not a single action. Peak process RSS observed during the run was 20.1 MiB for stdio and 63.3 MiB for the daemon. The fixture responses averaged 127 bytes for scrape and 2.2 KiB for the interact snapshot plus final scrape.

`agent-browser` v0.33.2 under Node 24 was excluded from the numeric comparison because repeated `open → snapshot` runs did not produce 30 valid samples in this environment; intermittent snapshots returned `(empty page)`. Failed samples were not counted.

> 🛡️ **Defensive Fallback Strategy:** Draco is designed for ultra-low latency and token efficiency. For heavily protected enterprise sites requiring a real browser, pair Draco with [Microsoft Playwright MCP](https://github.com/microsoft/playwright-mcp) as a high-fidelity rendering fallback, maintaining a strict separation of concerns.

### 1. Eliminating Selector Guesswork (Observation-First / Action-by-Ref)
* **The Pain:** AI models often fail at writing or compiling fragile CSS selectors to click buttons or type into inputs.
* **The Solution:** Draco implements a Playwright-class **Accessibility Snapshot** (`draco_interact_snapshot`). It serializes a semantic A11y tree (role, name, checked, disabled, etc.) and assigns stable reference keys (`e1`, `e2`, ...) directly to interactive nodes. Agents interact via references (`clickRef: "e1"`) instead of selector-string lottery.

### 2. Vue/React DOM Re-render Resilience (Ref Self-Healing)
* **The Pain:** Modern SPA frameworks dynamically destroy and recreate DOM nodes on state changes, immediately breaking hard pointers and throwing selector-not-found errors.
* **The Solution:** During serialization, Draco tracks sequential occurrence indices to form an identity triple `(role, name, nth)` for each ref. If a target node is unmounted/remounted, **Draco's page-side engine dynamically heals the reference** and clicks the correct recreated element.

### 3. Cutting Token Clutter (Interactive-Only & Promotion)
* **The Pain:** Injecting reference attributes on every tag in a deep HTML tree bloats the prompt, wastes tokens, and confuses the model.
* **The Solution:** Draco restricts references to core interactive roles by default. However, it dynamically **promotes** content nodes (like divs/spans) if they detect explicit `onclick` handlers, inline JS clicks, CSS `cursor: pointer` styles, or custom non-negative `tabindex` attributes. This keeps the tree compact without hiding actionable nodes.

### 4. Robust Failures & Charset Fidelity (CJK Sniffing)
* **The Pain:** Missing targets throw raw timeouts, and foreign-charset web pages (CJK: Traditional Chinese, Japanese, Korean) decode as U+FFFD (``) replacement garbage, blinding the LLM.
* **The Solution:** 
  - Failures are **self-describing** (`REF_NOT_FOUND` errors return the most recent A11y Snapshot as a dynamic `hint` to prompt immediate agent self-healing).
  - The fetch pipeline integrates **WHATWG encoding sniffing** (BOM → Content-Type → HTML Meta prescan → UTF-8 fallback) so CJK pages render flawlessly.

### 5. Adaptive Anti-Blocking Engine (Stealth Proxy & Jitter)
**Status: implemented in v0.23.0.**

* **The Pain:** Scraping rate-limited or WAF-protected sites can trigger 429 rate limiting or 403 blocks on cloud/datacenter IPs.
* **The Solution:**
  - **Header Emulation**: Automated generation of Desktop Chrome or Mobile Safari user-agents, alongside matching `Sec-Ch-Ua`, Platform, and Mobile flags.
  - **Humanized Jitter Delay**: Configurable random delay interval `[min, max]` per-domain, eliminating fixed request cycles.
  - **Transparent Proxy Rotation**: SOCKS5/HTTP proxy list support with automatic, transparent in-pipeline rotation and exponential backoff upon 429/403 blockages, fully invisible to the calling MCP agent.


---

## Configuration (`draco.toml`)

Draco reads `draco.toml` from the current working directory or `~/.config/draco/draco.toml` to configure stealth proxies, UA emulation, and domain-specific delays:

```toml
[stealth]
enabled = true
user_agent_mode = "desktop_chrome_random" # "desktop_chrome_random" | "mobile_safari" | "custom"
default_jitter_ms = [800, 2500]           # Random jitter interval [min, max]

[stealth.headers]
referer_emulation = true                 # Generate referrer based on target URL
sec_ch_ua_auto = true                     # Auto-generate matching Sec-Ch-Ua header

[proxy]
enabled = true
mode = "rotate_on_blocked"               # "rotate_on_blocked" (rotate upon 429/403) | "always_rotate"
max_retries = 3

# Proxy list (supports HTTP, HTTPS, SOCKS5)
endpoints = [
    "socks5://127.0.0.1:9050",
    "http://127.0.0.1:8888"
]

[domains."example.com"]
jitter_ms = [2000, 4500]                 # Domain-specific random delay
proxy_mode = "always_rotate"             # Rotate on every request for this domain
```

---

## MCP Server Configuration (`draco mcp` / `POST /mcp`)

Start Draco as an MCP stdio server:
```sh
draco mcp                        # stdio transport (newline-delimited JSON-RPC)
```

### Client Integrations

#### Claude Desktop / Code:
```json
{
  "mcpServers": {
    "draco": {
      "command": "draco",
      "args": ["mcp"]
    }
  }
}
```

#### opencode (`~/.config/opencode/opencode.json` — `{env:HOME}` is expanded; shell-style `$HOME` is not):
```json
{
  "mcp": {
    "draco": {
      "type": "local",
      "command": ["{env:HOME}/.draco/bin/draco", "mcp"],
      "enabled": true
    }
  }
}
```

### Provided Tools

* **`draco_scrape`** (`url`, `formats`, `selectors`, `tierMax`, `captureWindowMs`, `ignoreRobots`) — Scrapes a web page to Markdown, JSON-API, or CSS-selector matches (text + outer HTML).
* **`draco_discover`** (`url`, `tierMax`, `captureWindowMs`, `ignoreRobots`, `allowUnsafeReplay`) — Finds and replays the JSON API powering the page.
* **`draco_search`** (`query`, `limit`, `location`, `formats`) — Multi-engine web search with reciprocal-rank consensus and on-the-fly result scraping.
* **`draco_interact_open`** / **`close`** / **`exec`** / **`navigate`** / **`scrape`** — Stateful, cookie-aware browser sessions running inside an in-process V8 sandbox.
* **`draco_interact_snapshot`** — Generates a lightweight semantic accessibility tree snapshot.
* **`draco_interact_act`** — Dispatches sequential actions (`clickRef`, `typeRef`, etc.) via stable references with built-in self-healing.
* **`draco_interact_wait_for`** / **`dialogs`** / **`network_requests`** / **`console_messages`** — Waits for DOM conditions and inspects bounded session diagnostics.
* **`draco_interact_fill_form`** / **`navigate_back`** — Fills multiple controls through existing actions and navigates session history.

---

## Reference & Technical Documentation

For in-depth explanations, configuration options, and developers' guides, check out:

* 📄 **[CLI & REST API Guide](docs/usage.md)** — Detailed flags, environment options, and REST endpoints (Firecrawl-compatible `/v1/scrape`, `/v1/crawl`, `/v1/map`, `/v1/batch/scrape`).
* 📄 **[CSS Selector Extraction Spec](docs/select-format-spec.md)** — Detailed specification and E2E examples of the `--format select` extractor.
* 📄 **[MCP Agent Ergonomics Spec](spec/mcp-agent-ergonomics/spec.md)** — Design specs and requirements for the Observation-First / Action-by-Ref automation layer.
* 📄 **[Security Sandbox Model](docs/sandbox.md)** — Sandbox containment and V8 JIT isolation security architecture.
* 📄 **[Release & Development Runbook](AGENTS.md)** — Version rules, upstream synchronization process, and release pipelines.

---

## License

MIT OR Apache-2.0.
