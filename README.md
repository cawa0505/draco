# Draco (fork)

A fast, stealth, **native-Rust web scraper** — a lighter alternative to Firecrawl and Browserbase. Point it at a URL and get clean **Markdown + metadata** back, using a browser-faithful TLS/JA4 fingerprint to reach pages that block ordinary clients. No Node, no headless-Chrome fleet, no per-request browser boot.

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

While traditional scrapers output raw markdown or require heavy headless browsers (like Playwright) for interaction, Draco bridges the gap. It provides **browser-level agent-interaction capabilities without the browser-boot overhead**, operating directly in an in-process V8 sandboxed sandbox.

This fork specifically redesigns the MCP layer to solve the primary friction points of AI-driven automation:

### 1. Eliminating Selector Guesswork (Observation-First / Action-by-Ref)
* **The Pain:** AI models often fail at writing or compiling fragile CSS selectors to click buttons or type into inputs.
* **The Solution:** Draco implements a Playwright-class **Accessibility Snapshot** (`draco_interact_snapshot`). It serializes a semantic A11y tree (role, name, checked, disabled, etc.) and assigns stable reference keys (`e1`, `e2`, ...) directly to interactive nodes. Agents interact via references (`clickRef: "e1"`) instead of selector-string lottery.

### 2. Vue/React DOM Re-render Resilience (Ref Self-Healing)
* **The Pain:** Modern SPA frameworks dynamically destroy and recreate DOM nodes on state changes, immediately breaking hard pointers and throwing selector-not-found errors.
* **The Solution:** During serialization, Draco tracks sequential occurrence indices to form an identity triple `(role, name, nth)` for each ref. If a target node is unmounted/remounted, **Draco's page-side engine dynamically heals the reference** and clicks the correct recreated element.

### 3. Cutting Token Clutter (Interactive-Only & Promotion)
* **The Pain:** Injecting reference attributes on every tag in a deep HTML tree bloats the prompt, wastes tokens, and confuses the model.
* **The Solution:** Draco restricts references to core interactive roles by default. However, it dynamically **promotes** content nodes (like divs/spans) if they detect explicit `onclick` handlers, inline JS clicks, CSS `cursor: pointer` styles, or custom non-negative `tabindex` attributes. You get an ~80% leaner tree with 100% interactability.

### 4. Robust Failures & Charset Fidelity (CJK Sniffing)
* **The Pain:** Missing targets throw raw timeouts, and foreign-charset web pages (CJK: Traditional Chinese, Japanese, Korean) decode as U+FFFD (``) replacement garbage, blinding the LLM.
* **The Solution:** 
  - Failures are **self-describing** (`REF_NOT_FOUND` errors return the most recent A11y Snapshot as a dynamic `hint` to prompt immediate agent self-healing).
  - The fetch pipeline integrates **WHATWG encoding sniffing** (BOM → Content-Type → HTML Meta prescan → UTF-8 fallback) so CJK pages render flawlessly.

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

---

## Reference & Technical Documentation

For in-depth explanations, configuration options, and developers' guides, check out:

* 📄 **[CLI & REST API Guide](docs/USAGE.md)** — Detailed flags, environment options, and REST endpoints (Firecrawl-compatible `/v1/scrape`, `/v1/crawl`, `/v1/map`, `/v1/batch/scrape`).
* 📄 **[CSS Selector Extraction Spec](docs/select-format-spec.md)** — Detailed specification and E2E examples of the `--format select` extractor.
* 📄 **[MCP Agent Ergonomics Spec](spec/mcp-agent-ergonomics/spec.md)** — Design specs and requirements for the Observation-First / Action-by-Ref automation layer.
* 📄 **[Security Sandbox Model](docs/SANDBOX.md)** — Sandbox containment and V8 JIT isolation security architecture.
* 📄 **[Release & Development Runbook](AGENTS.md)** — Version rules, upstream synchronization process, and release pipelines.

---

## License

MIT OR Apache-2.0.
