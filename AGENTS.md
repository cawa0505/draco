# Draco (fork) — project rules

Fork of [0xchasercat/draco](https://github.com/0xchasercat/draco) at
[cawa0505/draco](https://github.com/cawa0505/draco). Read `docs/ROADMAP.md` and
`docs/select-format-spec.md` before touching pipeline code.

## Versioning (fork rule)

- Fork release version = **upstream latest minor + 1**. Upstream is at v0.20.x →
  our next release is v0.21.0. Never reuse an upstream tag name — our v0.19.0
  already collided with upstream's own v0.19.0 (different commits, same name).
- Bump `version` in root `Cargo.toml` (`workspace.package`); all crates inherit.
- Tag and release on `cawa0505/draco` only. Release CI runs on GitHub-hosted
  runners (fork-friendly) — do not restore the upstream self-hosted
  `namespace-profile-draco-*` runners; they do not exist on this fork.
- Release cadence: cut releases on our own milestones (feature complete /
  install demand), not synced to every upstream release — install.sh consumers
  pull our `releases/latest`. Before cutting, `git ls-remote --tags upstream`
  and set version = max(our next, upstream latest minor + 1).

## Upstream sync workflow

`upstream` remote is already configured. Per release:

1. `git fetch upstream` → measure delta: `git rev-list --count HEAD..upstream/main`
2. Review each upstream commit: **absorb** (wanted, e.g. fleet diagnostics) /
   **divert** (conflicts with our divergences) / **defer**
3. Merge or cherry-pick → run gates below → tag with the fork version rule

Deliberate divergences from upstream (keep when merging):
- `select` output format + `--selector` / `selectors` (docs/select-format-spec.md)
- Phase 4 (screenshot/a11y) waits for rustwright-core ≥0.2.0 — upstream declares
  screenshots a non-goal; our fork intentionally diverges
- Deploy: systemd user service example, `install.sh --from-source`, canonical
  owner URLs (`cawa0505`)
- Upstream process artifacts (`docs/superpowers/**` superpowers-generated plans):
  divert — Claude Code process noise, not code we adopt
- Specs follow the OpenSpec convention (`spec/` dir, see spec/README.md)

## Gates before ship

- `cargo fmt --all -- --check`
- `cargo check --no-default-features`
- `cargo build --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

After a source build, restart the systemd user service so the new binary is
active immediately: `systemctl --user restart draco.service` (binary at
`~/.draco/bin/draco`, service file `deploy/systemd/draco.service`).

## Deployment

- Daemon: systemd user service, `draco serve --port 3003` (3002 taken on this
  host). Override in `~/.config/systemd/user/draco.service.d/port.conf`.
- MCP: local/command mode (`draco mcp` stdio) in client configs; the daemon also
  binds `POST /mcp` (minimal). Remote MCP (Streamable HTTP) is Phase 0 of the
  roadmap. playwright-mcp (DockerSpace remote :3015) stays as the browser
  complement until Phase 4.
- install.sh downloads from `cawa0505/draco/releases/latest`; `--from-source`
  builds the repo and installs to `~/.draco/bin`.
