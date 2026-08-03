# MCP Agent Ergonomics — spec

Status: `draft` · Owner: fork (cawa0505) · Input for roadmap Phase 1 (interact)

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

## Out of scope / [待討論]

- Screenshots / real-browser fallback: roadmap Phase 4 (rustwright-core).
- Whether R1 lands in the live interact DOM (Phase 1) or the static scrape
  path too — decide when Phase 1 starts.
- R4 sourceTier on static scrape: additive wire field, confirm before adding.

## Verification

Per-requirement acceptance checks, exercised over the MCP stdio surface
(`draco mcp`), plus a smoke suite of the 7 requirements as MCP tool calls.
