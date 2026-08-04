# MCP Agent Ergonomics — spec

Status: `implemented` (Phase 1 shipped) · Owner: fork (cawa0505) · Input for roadmap Phase 1 (interact)

### Absorbed from agent-browser

identity triple (role, name, nth) for ref self-healing: deferred to follow-up
(see memory #2275). Snapshot serializer already computes role/name; nth-based
re-match can be layered on top without schema changes.

## Context

Draco's MCP surface already covers the Firecrawl-shaped scrape family
(scrape/map/crawl/batch/search/discover) plus interact. Upstream's scope ends
there. This fork's **main goal: strengthen the MCP layer for AI-agent use** —
the difference between "a tool an agent can call" and "a tool an agent can
drive reliably at low token cost".

Grounded in real agent usage of draco + playwright-mcp (2026-08).

## Requirements

### R1. Observation-first, action-by-ref
Agents fail by guessing (CSS selectors, DOM structure). Provide a page
observation view (a11y-style snapshot: role/name/state/text) and let actions
target **refs from the snapshot**, not hand-written selectors. Selector
targeting stays available as an escape hatch, never the default.
- Accept: snapshot returns stable per-node refs; act accepts refs.

### R2. Stateful sessions are explicit and inspectable
`interact` sessions: enumerate live sessions, report current URL/state/TTL,
auto-expire. Prefer stateless-by-default operations; session only when needed.
- Accept: `list_sessions` / `session_status` exist; orphaned sessions expire.

### R3. Bounded, deterministic, non-hanging calls
Every tool declares its budget (timeout, size cap, match cap) in the
description. No unbounded waits. Tier/cost control exposed to the agent.
- Accept: every MCP tool description lists limits; hangs become hard errors.

### R4. Parse-ready results with quality signals
Structured output (text/html/links/select separated) plus explicit quality
flags: JS rendered? (sourceTier), truncated?, robots respected?
- Accept: scrape results carry sourceTier + truncated flags consistently.

### R5. Self-describing failures
Errors say why + suggested next action (retry / upgrade tier / needs real
browser). Machine-readable error code + human hint.
- Accept: error responses carry code + hint fields.

### R6. Batch ergonomics
One call, N URLs, per-item results with per-item failures inline (not one
error killing the batch). Cap documented.
- Accept: batch tool already ships (25 cap) — keep, document in description.

### R7. Tool descriptions are the spec
MCP has no UI; the description field is what the agent reads to choose/use a
tool. Each description states: input schema, output shape, cost/tier,
failure modes — terse.
- Accept: description audit passes a review against this section.

## Phase 1 design — R1 snapshot + ref (in-progress, 2026-08-03)

Researched: happy-dom has NO a11y API (any version, incl. master) → role/name/
state computed manually (~150 lines JS in the isolate). Schema mirrors Playwright
MCP (`ariaSnapshot` AriaNode + `renderAriaTree` + `eN` refs) so agents parse both
the same way. Reference mapping: playwright `roleUtils.ts` / `ariaSnapshot.ts`.

### Wire contract (draco-types, additive — new interact response)

```rust
pub struct A11ySnapshot {
    pub url: String,
    pub nodes: Vec<A11yNode>,   // root-level tree
    pub refs: bool,             // true = refs mode: refs only on visible+interactive nodes
    pub truncated: bool,        // R4 signal: node cap or name-length cap hit
}

pub struct A11yNode {
    pub role: String,
    pub name: String,            // accname order; may be ""
    pub r#ref: Option<String>,   // "eN"; present only on interactable nodes in refs mode
    pub level: Option<u32>,      // heading/listitem/row/treeitem
    pub checked: Option<String>, // "true" | "false" | "mixed"
    pub disabled: Option<bool>,
    pub expanded: Option<bool>,
    pub selected: Option<bool>,
    pub pressed: Option<bool>,
    pub invalid: Option<String>,
    pub props: Option<A11yProps>, // {url, placeholder, value}
    pub children: Vec<A11yNode>,
}
```

### Rendered text (what the agent reads in MCP output)

Playwright-MCP `renderAriaTree` mirror, 2-space indent:

```
- button "Sign in" [ref=e12]
- textbox "Email" [ref=e13]: user@example.com
- heading "Results" [level=2] [ref=e14]:
  - list [ref=e15]:
```

### Serializer (JS in isolate, single-pass walk)

1. Hidden: drop STYLE/SCRIPT/NOSCRIPT/TEMPLATE; display:none / visibility:hidden
   (computed style); aria-hidden=true on self or ancestor (closest).
2. Role: explicit role= attr → else tag mapping (BUTTON→button, A[href]→link,
   INPUT[type]→textbox/checkbox/radio, H1-H6→heading+level, SELECT→combobox,
   NAV→navigation, …). role=none/presentation: keep only if focusable or carries
   ARIA states, else drop.
3. Name (accname order): aria-labelledby > aria-label > label/alt/value >
   content (only roles allowing name-from-content) > title > placeholder.
   Name-from-prohibited roles → "".
4. States per-role applicability: checked/disabled/expanded/selected/pressed/
   invalid (Playwright roleUtils lists).
5. Refs: eN counter, cached per element in a session WeakMap (stable across
   snapshots); assigned only when visible AND interactive
   (button/link/textbox/checkbox/radio/select/combobox/option/menuitem/tab/…).
6. Bounds: MAX_NODES (2000), name truncation (200 chars) → truncated=true.

### Ref targeting

`act` gains optional `ref` (e.g. "e12"); resolve via session WeakMap → element →
existing Action dispatch (Click/Type/Press/Scroll/Select/Hover/Wait unchanged).
Selector targeting stays as escape hatch. Unknown ref → R5 error: code
`REF_NOT_FOUND`, hint "ref not in current snapshot — page may have navigated;
re-snapshot".

### MCP surface

- New tool `interact_snapshot(sessionId)` → {url, nodes, text, refs, truncated}.
- Existing act tool gains `ref` param.
- R7: tool descriptions document snapshot format + ref semantics + bounds.

### Acceptance (R1)

1. open fixture → snapshot: correct roles/names/states per mapping; refs on
   visible+interactive nodes only.
2. Refs stable across consecutive snapshots (no page change).
3. act(ref=eN) performs the action; next snapshot reflects the change
   (e.g. click toggles expanded).
4. act(ref=unknown) → REF_NOT_FOUND with hint.
5. Snapshot bounded: caps enforced, truncated flag set when hit.
6. act(selector=…) still works (escape hatch).

### Blast radius / files (~7)

draco-types (types) · draco-runtime session.rs (Command::Snapshot + serializer
JS + WeakMap) · draco-core (Action ref field, open_interact_snapshot) ·
draco-cli serve/interact.rs (snapshot handler + act ref) · draco-cli mcp.rs
(tool + descriptions) · tests · spec.

## Out of scope / [待討論]

- Screenshots / real-browser fallback: roadmap Phase 4 (rustwright-core).
- Whether R1 lands in the live interact DOM (Phase 1) or the static scrape
  path too — RESOLVED (2026-08-03): live interact DOM in Phase 1; static
  scrape-path snapshot stays [待討論] for a later phase.
- R4 sourceTier on static scrape: additive wire field, confirm before adding.

## Verification

Per-requirement acceptance checks, exercised over the MCP stdio surface
(`draco mcp`), plus a smoke suite of the 7 requirements as MCP tool calls.
