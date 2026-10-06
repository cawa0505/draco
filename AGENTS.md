# Draco — project rules

Autonomous project maintained at [cawa0505/draco](https://github.com/cawa0505/draco). Formerly forked from `0xchasercat/draco` (upstream deleted/archived 2026-08). Read `docs/roadmap.md` and `docs/select-format-spec.md` before touching pipeline code.

## Main goal

Strengthen the MCP layer for AI-agent use — see `spec/mcp-agent-ergonomics/spec.md`
(R1 observation-first action-by-ref, R2 explicit sessions, R3 bounded calls,
R4 quality signals, R5 self-describing failures, R6 batch, R7 descriptions-as-spec).
Interact work rides roadmap Phase 1.

## Versioning & Repositories

- Canonical repository: `https://github.com/cawa0505/draco` (remote: `origin`)
- Versioning: Standard SemVer independently versioned. Next minor milestone is v0.25.0 (`plugin-system`), followed by v0.27.0 (`terminal-qr-stream`).
- Bump `version` in root `Cargo.toml` (`workspace.package`); all crates inherit.
- Tag and release on GitHub releases.

## Specifications & Divergences

Specs follow the OpenSpec convention (`spec/` dir, see `spec/README.md`):
- `select` output format + `--selector` / `selectors` (docs/select-format-spec.md)
- Phase 4 (screenshot/a11y) waits for rustwright-core ≥0.2.0
- Deploy: systemd user service example, `install.sh --from-source`
- Active/Planned Specs in `spec/`: `mcp-agent-ergonomics`, `stealth-proxy`, `plugin-system`, `terminal-qr-stream`

## Gates before ship

- `cargo fmt --all -- --check`
- `cargo check --no-default-features`
- `cargo build --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

**Strict compilation policy (zero warnings)**:
- All compiler warnings must be eliminated during compilation and checks such as `cargo check`, `cargo build`, and `cargo clippy`.
- Do not leave unused variables, dead code, unused functions, invalid imports, or unused constants. Remove dead code introduced by refactoring immediately, or explicitly mark it when that is intentional.
- Compiler and Clippy warnings are strictly forbidden in commits. Every warning must be fixed before committing so that compilation is 100% clean.

After a source build, restart the systemd user service so the new binary is
active immediately: `systemctl --user restart draco.service` (binary at
`~/.draco/bin/draco`, service file `deploy/systemd/draco.service`).

## Deployment

- Daemon: systemd user service, `draco serve --port 3003` (3002 taken on this
  host). Override in `~/.config/systemd/user/draco.service.d/port.conf`.
- MCP: local/command mode (`draco mcp` stdio) in client configs; the daemon also
  binds `POST /mcp` (minimal). Remote MCP (Streamable HTTP) is Phase 0 of the
  roadmap. Microsoft Playwright MCP stays as an optional external browser
  complement until Phase 4; no local deployment details belong in this repo.
- install.sh downloads from `cawa0505/draco/releases/latest`; `--from-source`
  builds the repo and installs to `~/.draco/bin`.

## Feature Specifics

### v0.23.0: Stealth Proxy & Adaptive Rate Limit Engine
- **Header Emulation**: Desktop Chrome & Mobile Safari UA randomized selection + matching Sec-Ch-Ua generation.
- **Jitter Delay**: Configurable random delay interval [min, max] matching the domain configuration.
- **Proxy Rotation**: Transparent HTTP/SOCKS5 proxy rotation with exponential backoff on 429/403 or network failures.
- **TOML Configuration**: Core system reads `draco.toml` from cwd or `~/.config/draco/draco.toml`.

### Plugin framework protocol

This repository defines a neutral plugin framework for extending Draco's request pipeline.

**Framework scope**
- This public repository provides only the plugin framework, lifecycle contracts,
  SDK/API, registry, and integration tests.
- Do not mention, document, reference, or encode the name, repository path,
  behavior, credentials, or implementation details of any external, private,
  proprietary, or planned plugin anywhere in this repository.
- Examples and fixtures must use neutral names such as `TestPlugin` and generic
  actions; they must not imply a specific product or private use case.

**Technical constraints**
- The Rust SDK contract lives in a small, dependency-light
  `draco-plugin-api` workspace crate so plugin crates do not depend on
  `draco-core` and the dependency graph remains one-way.
- Initial loading is compile-time registration only. Do not expose Rust trait
  objects through `.so` / `.dylib`; Rust has no stable trait-object ABI.
- Dynamic loading is a separate future design requiring a stable ABI boundary,
  version negotiation, and crash isolation. No dynamic loader scaffolding is
  included now.

**Core responsibilities**
- Core owns deadlines, cancellation, retries, resource limits, validation, and
  conversion to structured MCP errors.
- Plugins may request actions but may not execute unbounded control flow.
- No registered plugins may preserve current behavior and avoid hook traversal
  on the hot path.

**Contract terms**
- The public API defines the `DracoPlugin` trait and `PluginRegistry`.
- Plugin lifecycle includes `pre_request`, `post_response`, and `on_dehydrate`
  hooks with `PluginAction` control flow.
- All MCP plugin failures satisfy R5 self-describing failures with stable
  `code` and actionable `hint`.
- External implementation code and credentials never enter this repository,
  artifacts, logs, fixtures, or CI.

**Loading model**
- Phase 1 uses compile-time registration. The contract lives in a small,
  dependency-light crate that plugin crates depend on; `draco-core` consumes
  the same contract, keeping the graph one-way.
- The registry needs heterogeneous trait objects; Phase 1 uses `async_trait`
  because native async traits aren't object-safe. No assumption of "zero-cost";
  it's an acceptance target measured during benchmarking.

**Future design notes**
- `[待討論]` Define cache interception semantics separately; caching does not fit
  cleanly into the three request/response/dehydrate hooks.

**Delivery roadmap**
- See `spec/plugin-system/spec.md` for detailed Phase 1-4 delivery plan.

**Acceptance criteria**
- No behavior change when no plugins are registered.
- Disabled-plugin overhead measured and must show no material regression against v0.23.0 (threshold finalized after baseline).
- All gates pass with zero warnings.

**Usage guidance**
- External plugin development follows the public `draco-plugin-api` contract.
- Do not modify this repository with plugin-specific code, configuration, or documentation.
- Follow the neutral API contract and use generic names in examples and fixtures.
