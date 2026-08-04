# Draco Technical Usage & API Guide

This document contains detailed instructions for manual builds, command-line arguments, API endpoints, daemon mode, stateful interact sessions, security configurations, and active development runbooks.

---

## 1. Build & Installation

### Prebuilt Release
The fastest way to install Draco is via the shell script:
```sh
curl -fsSL https://raw.githubusercontent.com/cawa0505/draco/main/install.sh | sh
```

### Build from Source
Build from the repository and install the binary directly to `~/.draco/bin` (which also automatically restarts the running `draco.service` systemd user unit if present):
```sh
./install.sh --from-source
```

### Manual Build
```sh
git clone https://github.com/cawa0505/draco && cd draco
cargo build --release
```

#### Build Prerequisites
Prerequisites for `wreq`'s BoringSSL + `bindgen`, and V8 (required for the optional JSON/Isolate modes): `cmake`, a C/C++ compiler, `clang`/`libclang`, `perl`, `pkg-config`.
- **Debian/Ubuntu:** `apt install build-essential cmake clang libclang-dev perl pkg-config`
- **Fedora:** `dnf install gcc gcc-c++ cmake clang clang-devel llvm-devel perl pkgconf`
- **macOS:** Xcode Command Line Tools + `brew install cmake`

---

## 2. Command Line Usage

```sh
# Default: URL → Markdown on stdout (great for piping)
draco scrape https://example.com > page.md

# Full envelope (markdown + metadata + trace) as JSON
draco scrape https://example.com --json --pretty

# Stealth + politeness
draco scrape https://example.com --proxy socks5://127.0.0.1:9050 --delay 500
```

Exit codes: `0` success · `1` error · `2` unsupported · `3` needs_browser.

---

## 3. Advanced Scrape Tiers

### Optional: JSON-API Extraction (`--format json`)
Beyond Markdown, Draco can extract the **structured data an SPA loads from its own API** — a power feature for data-driven sites. It escalates through the cheapest tier that yields data:

1. **Static embedded state** — `__NEXT_DATA__`, JSON-LD, `window.__NUXT__`.
2. **Next.js build-id replay** — fetch `/_next/data/<buildId>/…​.json` directly.
3. **Runtime interception** — boots an **in-process V8 isolate** (restored from a build-time DOM-engine snapshot in single-digit milliseconds, JIT on), lets the page's JS hydrate, intercepts the `fetch`/`XHR` it fires for its data, ranks the intercepts, and replays the winner with the stealth client. The isolate is a *discovery oracle*, not a renderer — data requests are answered with a synthetic stub, and page JS has no host bindings: it cannot perform I/O.

```sh
draco scrape https://app.example.com --format json --pretty       # data[]
draco scrape https://app.example.com --format json --extract '$.props.pageProps'
draco scrape https://app.example.com --format both                # markdown + data
```

### Optional: CSS-Selector Extraction (`--format select` / `--selector`)
If you want to extract specific DOM elements rather than the whole page's Markdown or entire API payloads, use the `select` format with the `--selector` parameter:

```sh
# Extract specific CSS selector elements
draco scrape https://news.ycombinator.com --format select --selector ".titleline > a"
```
Each match in the returned array contains:
- `text` — the whitespace-collapsed inner plain text of the element.
- `html` — the raw `outerHTML` of the element.

Matches are automatically capped at 1000 per request to prevent payload blowup.

### Client-rendered SPAs → Markdown (render-then-Markdown)
Some pages render their *content* only after JavaScript runs — the fetched HTML is a thin shell (an empty `<div id="root">`). Draco handles these automatically: when the initial parse finds almost no content and Tier 2 is permitted (the default), it hydrates the shell in the same in-process V8 isolate, serializes the **live DOM**, splices the shell's real `<head>` (title / Open Graph / canonical) onto the hydrated `<body>`, and re-runs the exact same content engine over it. You get clean Markdown from a client-rendered page with no headless browser — the trace shows a `runtime.render` step and `source_tier: runtime_interception`.

```sh
draco scrape https://spa.example.com            # thin shell → hydrated Markdown
draco scrape https://spa.example.com --tier-max 1   # opt out: static shell only
```

---

## 4. Persistent Daemon (`draco serve`)

Run Draco as a **persistent HTTP daemon with a Firecrawl-compatible REST API** — the process stays warm (no per-scrape binary spawn), and existing Firecrawl clients can point at it unchanged:

```sh
draco serve                    # http://127.0.0.1:3002 (Firecrawl's default port)
draco serve --host 0.0.0.0 --port 8080 --max-concurrency 16
```

### POST `/v1/scrape`
```sh
curl -X POST http://127.0.0.1:3002/v1/scrape \
  -H 'content-type: application/json' \
  -d '{"url": "https://spa.example.com", "formats": ["markdown"]}'
# → { "success": true, "data": { "markdown": …, "metadata": { "title", "sourceURL", … } } }
```
- `formats`: `"markdown"` (default) and/or `"json"` (the tiered JSON-API extraction, under `data.json`). Formats Draco doesn't produce yet (`html`, `rawHtml`, `links`, `screenshot`) are rejected with a clear `400`.
- Unknown Firecrawl fields (`onlyMainContent`, `waitFor`, …) are accepted and ignored; failures use the `{ "success": false, "error": … }` envelope (`502` upstream/network, `422` unsupported target, `400` bad request).
- Draco extensions per request: `tierMax`, `captureWindowMs`, `noJail`, `allowUnsafeReplay`, `ignoreRobots`, `proxy` — plus `timeout` (Firecrawl's).
- `GET /health` → `{ "status": "ok", "version": … }`.

### POST `/v1/map`
Fast site URL discovery: merges `/sitemap.xml` (sitemap indexes followed one level) with the page's own links; same-host filtered, deduped, `search`-filtered, `limit`-capped.
```sh
curl -X POST localhost:3002/v1/map -H 'content-type: application/json' \
  -d '{"url": "https://docs.example.com", "search": "guide"}'
# → { "success": true, "links": [ … ] }
```

### POST `/v1/crawl`
Async crawl jobs: a bounded same-host BFS (`limit` default 10, cap 100; `maxDepth` default 2) where every page runs the full extraction ladder. Poll `GET /v1/crawl/{id}` for `{ status, total, completed, data: [ per-page results ] }`.

### POST `/v1/batch/scrape`
Scrape a list of URLs as one async job. Scrape options are **flat** at the top level (`formats`, `includeTags`/`excludeTags`, `headers`, `waitFor`, …), applied to every URL.
```sh
curl -X POST localhost:3002/v1/batch/scrape -H 'content-type: application/json' \
  -d '{"urls": ["https://a.example", "https://b.example"], "formats": ["markdown"]}'
# → { "success": true, "id": "7", "url": "/v1/batch/scrape/7" }
```

### Webhooks
Crawl and batch requests accept a `webhook` (a bare URL string or `{ url, headers, metadata, events }`). The job fires `started`, `page` (scraped document), `completed`, and `failed` events.

---

## 5. Stateful Interact Sessions

Agents can drive stateful browser-like interactions against the sandboxed V8 DOM engine. Cookies are persisted across the session lifetime.

* **`POST /v1/interact/open`** — Starts a session.
* **`POST /v1/interact/<id>/exec`** — Evaluates an async function body.
* **`POST /v1/interact/<id>/navigate`** — Navigates to a new page (cookie-aware).
* **`POST /v1/interact/<id>/scrape`** — Returns Markdown/HTML of the live DOM.
* **`DELETE /v1/interact/<id>`** — Closes the session.

---

## 6. Project Architecture

### Workspace layout

| Crate | Role |
|-------|------|
| `draco-types` | Wire + result contract (no I/O) |
| `draco-net` | Stealth TLS/JA4 HTTP client (wreq/BoringSSL): cookie jar, proxy, robots, backoff |
| `draco-static` | **Markdown + metadata extraction** (Firecrawl-parity) · JSON embedded-state · build-id replay |
| `draco-runtime` | Tier 2 **in-process V8 isolate** (JIT): real happy-dom DOM engine baked into a build-time V8 snapshot; `fetch`/`XHR` interception; Observe/Render fetch modes; concurrent async chunk loading |
| `draco-core` | Escalation state machine, challenge short-circuit, ranking, replay, chunk cache |
| `draco-cli` | The `draco` CLI + output contract |

### Feature flags

- **default (`tier2`, `serve`)** — everything: the V8 isolate for `--format json` runtime interception / render-then-Markdown, plus the `draco serve` daemon.
- **`serve`** — the persistent HTTP daemon (axum). Independent of `tier2`: `--no-default-features --features serve` exposes the same REST API with the ladder capped at the static tiers.
- **`--no-default-features`** — a lean build with **no V8/axum linked**. Smaller binary, faster build.
```sh
cargo build -p draco-cli --no-default-features   # lean, V8-free, axum-free
```

---

## 7. Security & Isolations

### Security Model (Tier 2 only)
Markdown scraping of a static page executes no page JavaScript. Tier 2 (runtime interception / render-then-Markdown) does. Draco's containment is the **V8 isolate itself**: the context has **no host-capability bindings** — the only ops exposed to page JS record an intercepted request, load a script chunk, log a diagnostic, sleep, and resolve URLs. There is no network, filesystem, or process access. JIT is on (`--single-threaded`). This is the same class of isolation Puppeteer/Playwright/jsdom rely on, works identically on macOS and Linux, and needs zero configuration.

### Challenge walls
Draco does **not** defeat JS challenge walls (Cloudflare/DataDome/…); a genuine interstitial (blocking status + real challenge page) short-circuits to `needs_browser`.

---

## 8. Platforms

| Platform | Markdown scrape | JSON Tier 0/1 | Tier 2 isolate |
|----------|:---:|:---:|:---:|
| **Linux** `x86_64-gnu` | ✅ | ✅ | ✅ |
| **macOS** `aarch64-darwin` | ✅ | ✅ | ✅ |

---

## 9. Development Runbook

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

On a memory-/disk-constrained box (CI containers, the ~4 GiB build sandbox), run the gates through the guarded wrapper instead:
```sh
bash scripts/gate.sh            # fmt + clippy + test, disk-guarded
bash scripts/reclaim.sh         # free regenerable build artifacts in a pinch
```
See **[docs/sandbox.md](docs/sandbox.md)** for the full runbook.
