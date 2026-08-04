# v0.25.0 Extensible Plugin Architecture

**Status:** approved proposal  
**Target:** `draco-core` and MCP-facing pipeline  
**Prerequisites:** v0.23.0 benchmark and README update complete

## Goal

Keep Draco's core focused on HTTP transport, DOM dehydration, accessibility
semantics, and charset-safe text processing while allowing optional behavior to
intercept the request pipeline through a neutral public contract.

This repository defines only the framework. It must not identify or describe
any external or private plugin, project, product, or implementation.

## Pipeline

```text
request
  -> pre_request
  -> network execution
  -> post_response
       -> continue | retry | custom action | abort
  -> on_dehydrate
  -> MCP result
```

Plugins run in registration order. The first control-flow action other than
`Continue` stops the current hook chain. Core owns retry limits, timeouts,
cancellation, and final error conversion so a plugin cannot create an unbounded
loop or bypass MCP R5 self-describing failures.

## Proposed API

```rust
#[async_trait::async_trait]
pub trait DracoPlugin: Send + Sync {
    fn name(&self) -> &'static str;

    async fn pre_request(&self, request: &mut reqwest::Request)
        -> Result<(), DracoError>;

    async fn post_response(&self, response: &ResponseContext<'_>)
        -> Result<PluginAction, DracoError>;

    async fn on_dehydrate(&self, content: &mut String)
        -> Result<(), DracoError>;
}

pub enum PluginAction {
    Continue,
    Retry(RetryRequest),
    Custom(CustomAction),
    Abort(String),
}
```

Default hook bodies return `Ok(())` or `Continue`. `PluginRegistry` stores the
ordered plugin chain and exposes one runner per lifecycle hook.

## Required boundaries

- No plugin enabled means no hook traversal in the request hot path.
- Plugin execution shares the caller's deadline and cancellation.
- Core validates retry requests, custom action payloads, and output size at the
  trust boundary.
- Plugin errors become structured Draco errors with MCP `code` and `hint`.
- A plugin may request control flow; only core executes retries and network
  transitions.
- Response inspection uses a bounded `ResponseContext`, not a consumable
  `reqwest::Response` body shared across plugins.
- Existing charset sniffing remains a core invariant and cannot be disabled by
  an `on_dehydrate` plugin.
- Public core exposes contracts only. External implementations and credentials
  stay outside this repository.

## Loading model

Phase 1 supports compile-time registration from Rust crates. The public contract
lives in a small, dependency-light `draco-plugin-api` workspace crate. Plugin
crates depend on that API crate; `draco-core` consumes the same contract, keeping
the dependency graph one-way.

The registry needs heterogeneous trait objects. Native `async fn` in traits is
not object-safe for this use, so Phase 1 uses `async_trait`. Its boxed-future
cost must be measured; “zero-cost” is an acceptance target, not an assumption.

Rust trait-object ABI is not stable, so v0.25.0 does not load `.so` / `.dylib`
plugins. Dynamic loading is a separate future design requiring a stable ABI,
version negotiation, resource limits, and crash isolation. No speculative
dynamic-loader scaffolding is included now.

- `[待討論]` Define cache interception semantics separately; caching does not fit
  cleanly into only the three request/response/dehydrate hooks.

## Delivery roadmap

### Phase 1 — Trait and registry

- Define plugin context, actions, errors, and ordered registry.
- Add one minimal test plugin proving ordering, interception, and default hooks.
- Benchmark disabled-plugin overhead against v0.23.0.

### Phase 2 — Core pipeline integration

- Add lifecycle calls at the single shared request/dehydrate flow.
- Enforce deadline, retry budget, output bounds, and structured failures.
- Verify CLI, daemon REST, MCP stdio, and `POST /mcp` inherit the same chain.

### Phase 3 — Existing optional behavior migration

- Move eligible optional behavior behind the neutral plugin contract.
- Keep retry execution in core so control flow is not duplicated.
- Preserve existing `draco.toml` behavior and observable output.
- Compare enabled and disabled performance with the v0.23.0 baseline.

### Phase 4 — External integration proof

- Build a neutral external test crate outside this repository.
- Register it through the public compile-time API.
- Verify bounded actions, timeout, cancellation, and isolation using a local
  fixture.
- Publish only contract-level evidence using generic names and data.

## Acceptance criteria

- No behavior change when no plugins are registered.
- Disabled-plugin benchmark shows no material regression against v0.23.0; the
  concrete threshold is `[待討論]` after the baseline is finalized.
- Plugin ordering and first-intercept semantics are deterministic and tested.
- Retry and custom-action paths cannot exceed the core deadline or retry cap.
- All MCP plugin failures satisfy R5 with stable `code` and actionable `hint`.
- External implementation code and credentials never enter this repository,
  artifacts, logs, fixtures, or CI.
- All repository gates pass with zero warnings.
