# Roadmap — playwright-mcp / ax-mcp parity

目標：把 ax-mcp 部署（remote MCP + 瀏覽器工具）與 playwright-mcp 功能引進 Draco，
分階段、每階段有驗證 gate。原則：DOM-only 的留在 in-process V8 + happy-dom（零新依賴）；
需要真實 layout/paint 的另開 opt-in 引擎，不偷渡進無瀏覽器路線。

## 規格來源：ax-mcp（`~/DockerSpace/ax-mcp`）

現有部署 = supergateway 把兩個 stdio MCP 橋成 remote Streamable HTTP `/mcp`：
- `ax-mcp`（3014）：包 yusukebe/ax（Bun+linkedom 非瀏覽器爬蟲）→ 工具 `scrape_web`
- `playwright-mcp`（3015）：官方 mcp/playwright 映像（Chromium，stateful）

**`scrape_web` 契約（url, selector, format: text|json, wait）→ Draco 已 1:1 覆蓋**
（`draco_scrape(url, formats, selectors, waitFor)` 是 superset — ax 不會跑 JS，Draco 會）。
所以 ax 那一半**已完成**，不必再實作。

**真正缺的是 deployment 模式**：remote MCP。Draco daemon 已有 `POST /mcp`
（最小 Streamable-HTTP subset：單 JSON-RPC POST → 單 application/json，無 SSE stream、
無 transport session）。opencode `type: "remote"` 走 Streamable HTTP（GET SSE + POST session），
目前只能 fallback。目標：一個 `draco serve` 取代 ax-mcp + playwright-mcp 兩個容器。

## 現況對照（playwright-mcp parity gap）

| playwright-mcp tool | Draco 現況 | 差距 |
|---|---|---|
| `browser_navigate` / `navigate_back` | `interact_navigate` | 缺 back（無 history stack） |
| `browser_snapshot`（a11y tree + ref） | 只有 markdown/html serialize | **缺 a11y snapshot — 最大差距** |
| `browser_click/type/press/hover/select_option` | `interact_act`（CSS selector 定位） | 缺 role/name ref 定位 |
| `browser_fill_form` | 逐欄 type | 缺 batch sugar |
| `browser_evaluate` | `interact_exec`（expression 形態） | 已對等 |
| `browser_wait_for`（text/visible/time） | 只有 selector-wait / 固定 ms | 缺 text/visible wait |
| `browser_handle_dialog` | 無 | 缺 alert/confirm/prompt 攔截 |
| `browser_network_requests` / `console_messages` | 只有 capture 內一次性 trace | 缺 session 累積工具 |
| `browser_tabs` | 單 document per session | 缺多 tab |
| `browser_drag`/`drop`/`file_upload` | 無 | 缺（可 event 模擬，低價值） |
| `browser_take_screenshot` | 無 | **缺 — 需真實 render 引擎** |
| `browser_resize` | 無 | 缺（happy-dom 只能 JS 層 emulation） |
| `browser_run_code_unsafe` | `interact_exec` | 已對等 |

## Phase 0 — remote MCP 完整化（ax-mcp 部署取代，最高價值）

`draco serve` 的 `POST /mcp` 升級為完整 Streamable HTTP：GET `/mcp` SSE stream +
POST session 語意（`Mcp-Session-Id`）、`Accept: text/event-stream` 尊重。
opencode `type: "remote"` / Claude remote 直接連 `http://host:3003/mcp`，不用 supergateway。

努力：1–2 天（現有 JSON-RPC 核心重用，補 transport 層）。
Gate：opencode remote MCP 連上 draco daemon，`draco_scrape` 遠端呼叫成功；
curl SSE GET + POST 都通。之後 ax-mcp 容器可退役（scraper 部分）。

## Phase 1 — a11y snapshot + ref 定位（DOM-only，agentic 關鍵）

playwright-mcp 能 agentic 靠 a11y tree + ref 定位；Draco 只有 CSS selector，LLM 得猜。

- [ ] `snapshot` op：DOM → accessibility tree（role/name/state；ARIA + HTML 語意對照表）
- [ ] MCP `draco_interact_snapshot`（depth/boxes 參數）
- [ ] act 支援 ref/role 定位（`role:name` → CSS selector，經 a11y tree）
- [ ] scrape 加 `--format accessibility`（一次性 snapshot，與 interact 共用建構）

努力：1.5–2 天（role table 是主要量）。無新依賴。
Gate：fixture 頁 snapshot → role tree 正確、ref 定位點中目標；`--format accessibility` 對照 DOM。

## Phase 2 — session 工具補齊（DOM-only，每項獨立可並行）

- [ ] `wait_for` 擴充：text-match / visible
- [ ] dialog 攔截：glue JS 覆寫 `window.alert/confirm/prompt`，記錄 + 自動回應
- [ ] network viewer：session 累積 brokered fetch 浮成 `interact_network_requests`
- [ ] console viewer：session 累積 console 浮成 `interact_console_messages`
- [ ] `fill_form` batch sugar
- [ ] `navigate_back`（兩層 history stack）

努力：每項 0.25–0.5 天，共約 1.5 天，可拆並行。
Gate：單 session 內「點擊 → 填表 → 等 text → 讀 network/console」連鎖驗證。

## Phase 3 — 多 tab（架構改動）

session 持 page map `{id → isolate document}`，切換換 active document。動 session 核心。
努力：1–2 天。低優先，parity 補齊非痛點。
Gate：雙 tab 獨立 JS 狀態、切換後 serialize 正確。

## [待討論] Phase 4 — 瀏覽器引擎 opt-in（screenshot / 真實 layout）

happy-dom 無 layout/paint — screenshot 是跨不過的硬邊界。Rust 選項（2026-08 查證）：

| 方案 | runtime 依賴 | 判決 |
|---|---|---|
| **chromiumoxide 0.9.1** | 純 Rust CDP，只需 chromium binary | **建議**。a11y 域齊全（`getFullAXTree`/`queryAXTree` 走 `Page::execute`，~30 行 wrapper）；fetcher 可取 ChromeHeadlessShell（~88MB zip）或重用既有 chrome；截圖 high-level API 現成 |
| rustwright-core 0.1.x | 純 Rust，Skyvern 出品 | 觀望 — alpha（2026-07 才出），已內建 Playwright 級 a11y + 截圖；出 0.1 後 revisit |
| playwright-rs 0.15.1 | **Node 18+ + driver + chromium** | 出局 — Node runtime 破壞單一 static binary 部署 |
| wry / webkit2gtk | 系統 webview，零下載 | 出局 — per-OS、需 display，layout 隨系統瀏覽器，不可預期 |

設計：`--browser-engine chromiumoxide` opt-in feature；default 路徑保持零瀏覽器。
引擎作用：screenshot（element/fullPage）、真實 layout 互動、CDP 級 a11y（Phase 1 的 fidelity 升級）。
部署：取代 docker 化 playwright-mcp 容器（3015）— 一個 daemon 全包。
努力：3–5 天。成本：chromium binary ~88–160MB + 系統 libs（libnss3/libgtk-3/libgbm/fonts），
feature-gated 不影響 distroless 部署。

## Out of scope

- 完整 Playwright API 相容（`exec` 已對等 `run_code_unsafe`，其餘不追）
- ax 本身（Draco 已 superset；`scrape_web` 契約直接由 `draco_scrape` 承接）
- 真實瀏覽器 fingerprint / anti-bot（安全模型設計邊界，非缺口）
