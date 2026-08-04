# Specs — OpenSpec convention

本 fork 的規格遵循 [OpenSpec](https://github.com/github/openspec) 慣例（純 markdown，
無需 CLI，agent 與人皆可讀）。既有 legacy specs 不動，新規格一律進 `spec/`。

## 結構與生命週期

- `spec/<name>/spec.md` — 規格本體，status 標在標題或 front-matter：
  `draft` → `in-progress` → `implemented`（或 `cancelled` / `deferred`）
- 未決的 scope 項內嵌 `[待討論]` marker，保留決策痕跡（沿用本 fork 慣例）
- 可選：`spec/<name>/tasks.md` 任務拆解（對應 OpenSpec tasks）
- `implemented` 的 spec 在文末記錄驗證證據（tests / 命令輸出摘要）

## Legacy specs（不搬移）

- `docs/select-format-spec.md` — select format，implemented (v1)
- `docs/roadmap.md` — ax、Playwright MCP、agent-browser 的能力邊界與階段化 roadmap

## 下一個 OpenSpec 標的

- `plugin-system/spec.md` — v0.24.0 microkernel plugin pipeline，approved proposal
