# Select Format Specification — CSS Selector Extraction

Status: **implemented (v1)** · Area: Wire Contract (`draco-types`), Core (`FormatSet`/`Config`/`machine`), Static (`selector` engine), CLI, Daemon, MCP.

---

## 1. Motivation

The ability to extract specific DOM elements based on a CSS selector and retrieve their raw text or HTML is a highly-requested feature for fine-grained scraping, historically matching capabilities of older libraries (like `ax-scraper`). 

While Draco already provides schema-driven structured JSON extraction (`--extract` via JSON Schema), this specification introduces a lightweight, raw selector mode: given one or more CSS selectors, retrieve the matched elements' whitespace-collapsed text and raw outerHTML directly.

---

## 2. Design

This feature introduces a new output format `select` paired with a repeatable CLI flag `--selector <CSS>` (which maps to `Config.selectors: Vec<String>`).

### Rules & Behaviors
- **Implicit Format**: Providing `--selector` automatically implies the `select` format (you do not need to explicitly declare `--format select`).
- **Format without Selectors**: Specifying `--format select` without providing any `--selector` parameter will reject the request with `exit 1` (CLI) or a `400 Bad Request` (Daemon).
- **No Matches**: If a selector matches nothing, it returns `matches: []` rather than throwing an error.
- **Invalid Selectors**: An invalid CSS selector triggers an immediate request-level rejection (`DracoError::Config`). It does not fail silently, as it represents a direct user configuration error.

### Wire Contract
An additive, optional field is added to `ExtractionResult` (matching the level of `links` and `endpoints`). This field is omitted from serialization when `None`:

```json
"selector": [
  {
    "selector": "div.article",
    "matches": [
      { "text": "Deals of the day", "html": "<div class=\"article\">…</div>" }
    ]
  }
]
```

- Matches are grouped per input selector, retaining search order.
- Each match is capped at `1000` (reusing `extract_schema::MAX_MATCHES`).
- `text`: Whitespace-collapsed plain text (reusing `extract_schema::collapse_ws`).
- `html`: Raw **outerHTML** of the matched element. It is left unaltered (no href absolute resolution or styling sanitization) to preserve local fidelity.

---

## 3. Execution Pipeline (Static & Hydrated)

The `select` format mirrors the extraction lifecycle of the `html` format:
- **Static Pipeline**: Executes against the initial static HTML document (post `filter_body` in `machine.rs`) alongside include/exclude tag parameters.
- **Render Pipeline**: For SPAs, the hydrated DOM is queried directly inside the V8 isolate once rendering is complete, updating the `selector` field dynamically.
- **Format Integration**: `FormatSet.select` is included in `wants_static_content()` to ensure the fetch engine fetches static HTML when no JS hydration is requested.
- **Trace & Telemetry**: The pipeline records a `static.select` or `runtime.render.select` trace step reporting the total number of matched nodes.

---

## 4. Interfaces & Integration Gaps

| Interface | v1 Status | Details & Notes |
|---|---|---|
| **CLI (`draco scrape`)** | ✅ Implemented | `--format select --selector "..."` (Repeatable) |
| **Daemon (`POST /v1/scrape`)** | ✅ Implemented | `formats: ["select"]` + `selectors: ["..."]` (String or Array) |
| **MCP (`draco_scrape` tool)** | ✅ Implemented | Added `selectors` property to MCP tool schema. |
| **Crawl / Batch / Search** | `[Pending]` | Requires updates to `PageQuery` schemas; deferred to Phase 2. |
| **Interact Scrape** | `[Pending]` | Live interact session selector querying is scoped for Phase 2. |

---

## 5. Blast Radius & Files Touched

1. **`crates/draco-types/src/lib.rs`** — Added `ExtractionResult.selector: Option<Vec<SelectorMatch>>` and `SelectorMatch { selector, text, html }`.
2. **`crates/draco-core/src/lib.rs`** — Added `FormatSet.select` and `Config.selectors`.
3. **`crates/draco-core/src/machine.rs`** — Configured the static and hydrated DOM extraction and trace endpoints.
4. **`crates/draco-core/src/interact.rs`** — Initialized `selector: None` in interact session results.
5. **`crates/draco-static/src/extract_schema.rs`** — Implemented helper `select_matches(html, selectors) -> (Vec<SelectorMatch>, Vec<String>)` with collapsing whitespaces.
6. **`crates/draco-cli/src/main.rs`** — Registered `--selector` flag and validated formats.
7. **`crates/draco-cli/src/serve/mod.rs`** — Updated `/v1/scrape` request body parsing and Firecrawl compatibility mappings.
8. **`crates/draco-cli/src/mcp.rs`** — Registered selectors inside MCP tool descriptors.

---

## 6. Non-Goals

- Merging different selectors into a single unified array.
- Serializing matched nodes back to markdown (users can achieve this using `--include-tag` alongside standard markdown formatting).
- Selector support inside bulk crawling pipelines (deferred).

---

## 7. Verification & Tests

```sh
# Run targeted unit tests
cargo test -p draco-static -p draco-types
cargo test -p draco-cli -- serve::tests

# End-to-end CLI validation
draco scrape https://example.com --format select --selector "h1"
```
