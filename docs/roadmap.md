# Draco Roadmap

這份文件記錄 Draco fork 的實際演進，不把外部研究、upstream 變更與本 fork
自行開發的功能混寫成同一件事。已完成項目以版本號標示；尚未實作的項目保留
在後續階段，不以規劃文字冒充完成紀錄。

## 定位與外部參考

Draco 的主路線是 browserless-first 的 HTTP、DOM extraction 與 AI agent
互動工具。需要真實瀏覽器時採 escalation，而不是把 Chromium 強行放進核心。

| 專案 | 在本 roadmap 的角色 |
|---|---|
| [ax](https://github.com/yusukebe/ax) | 外部 AI agent CLI；原始專案不是 MCP server。只在 CLI 對 CLI 的能力與效能比較中參考。 |
| [Playwright MCP](https://github.com/microsoft/playwright-mcp) | Microsoft 的真實瀏覽器 MCP；作為 browser-only 互動、layout 與 screenshot 的外部互補方案。 |
| [agent-browser](https://github.com/vercel-labs/agent-browser) | Vercel Labs 的瀏覽器自動化 CLI，也可作為 stdio MCP；作為 Draco 的 browser fallback 參考。 |

這些專案是能力比較與 escalation 的來源，不代表 Draco 已經吸收其全部實作，
也不代表 Draco 依賴它們才能執行。

## Upstream 基礎與 fork 分流

本 fork 沿用 [0xchasercat/draco](https://github.com/0xchasercat/draco) 的
Firecrawl-compatible 主幹：scrape、map、crawl、batch、search、discover、webhook、
MCP 與 interact。每次 release 前會逐筆檢視 upstream commits，依需求標成：

- **absorb**：不破壞 fork 差異、能改善共同主幹的修正與診斷能力。
- **divert**：與本 fork 已完成的 select format、MCP ergonomics 或部署方式衝突，
  保留 fork 實作。
- **defer**：方向合理但尚未到實作優先級，留在 roadmap 而非預先搭架構。

Upstream 將 screenshot 與 browser automation 視為 non-goal；本 fork 同樣不以
happy-dom 偽造這些結果，但保留未來 opt-in real-browser escalation 的研究路線。
因此這是刻意分流，不是遺漏 upstream 功能。

## 版本演進

### v0.21.0 — fork 基礎與 extraction parity

基於 upstream v0.20.x，完成第一批 fork 維護工作：

- 加入 systemd user service 範例與 `install.sh --from-source`。
- 將 release workflow 改為 fork 可用的 GitHub-hosted runners。
- 將公開安裝與 release URL 統一指向 `cawa0505/draco`。
- 吸納外部 scraper 使用情境，加入 `select` output format 與重複的
  `--selector` / `selectors` 參數；這是 Draco 自行維護的 extraction 差異，
  不是把外部 CLI 當成 MCP 實作。
- 建立 MCP agent ergonomics 的規格與 DOM-only 互動路線。

同期完成的決策：以 observe → act → observe 的 agentic 互動迴圈作為主價值，
而非把 screenshot 當成核心目標；需要真實 layout 的能力留在 browser escalation。

### v0.22.0 — MCP Phase 1：snapshot 與 action-by-ref

這個版本吸納 agent-browser 使用情境中對 identity triple 的需求，並參考
Playwright MCP 的 accessibility snapshot / ref 互動模型；實作仍是 Draco 自己的
happy-dom/V8 serializer，不是 runtime 依賴外部瀏覽器。

已完成：

- `draco_interact_snapshot`：輸出 role、name、state、children、refs 與 truncation
  signal。
- `interact_act` 的 `clickRef`、`typeRef`、`pressRef`、`scrollRef`、
  `selectRef`、`hoverRef`、`waitRef`。
- `REF_NOT_FOUND` 結構化錯誤與可採取行動的 hint。
- identity triple `(role, name, nth)` 的 ref self-healing。
- interactive-only refs 與明確可點擊內容的 promotion，降低 snapshot token 雜訊。
- sessions、batch、quality signals 與 tool descriptions-as-spec。
- charset-aware response decoding，維持 BOM → Content-Type → meta → UTF-8
  fallback 的順序，避免 CJK 頁面產生 replacement characters。

`spec/mcp-agent-ergonomics/spec.md` 的 Phase 1 已標示為 `implemented`。
靜態 scrape 的 accessibility format 不在本次交付範圍，後續是否加入仍是
`[待討論]`。

### v0.23.0 — Stealth HTTP pipeline

這個版本是 fork 自行擴充的 HTTP pipeline，並保留 upstream 的相容性與品質 gate：

- Header emulation：Desktop Chrome / Mobile Safari 的 UA 與匹配的 client hints。
- Domain 等級的 jitter delay。
- HTTP/SOCKS5 proxy pool、429/403 與網路錯誤的 rotation、backoff、retry。
- `draco.toml` 設定整合。
- 完成 workspace gates、release build 與 v0.23.0 release。

效能數字只在測試條件固定、結果實際量測後才寫入 README；外部網站保證與
「up to」式宣稱不列入文件。

## MCP 與外部工具的實際邊界

目前 Draco 的主要 MCP 部署方式是 **local command mode：`draco mcp`（stdio）**。
Daemon 另有最小的 `POST /mcp` JSON-RPC endpoint，但尚未完成完整的
Streamable HTTP transport、SSE 與 transport session 語意。Remote MCP 因此仍是
未完成項目，不得寫成已取代外部部署的功能。

外部工具的正確分工如下：

- ax 是 CLI 參考，不是 Draco 的 MCP transport；ax 的 scraper 使用情境可與
  Draco 的 `draco_scrape` 做 CLI 對 CLI 比較。
- Playwright MCP 與 agent-browser 都是需要真實 Chromium 的 browser 工具，
  可補足 screenshot、真實 layout、OS-level file upload、overlay hit-testing
  等 DOM-only 無法可靠提供的能力。
- Draco 優先處理 HTTP extraction、可解析結果、a11y snapshot 與低成本互動；
  卡住時才升級到真實瀏覽器工具。

## 後續 roadmap

### Phase 0 — Remote MCP transport（未完成）

將 daemon 的最小 `POST /mcp` 擴充為完整 Streamable HTTP：GET SSE、POST
session、`Mcp-Session-Id` 與 `Accept` 語意。完成前，local stdio 仍是正式部署方式。

**驗證 gate：** remote MCP client 可連線，`draco_scrape` 可成功呼叫，SSE GET
與 POST session 都有 integration test。這只處理 transport，不等於引入瀏覽器。

### Phase 1 — A11y snapshot + ref（v0.22.0，已完成）

已完成內容見版本演進。保留一項尚未決定的延伸：是否讓靜態 scrape path 也輸出
accessibility format；目前 interact live DOM 是唯一正式 snapshot path。

### Phase 2 — Session 工具補齊（已實作，未發布）

已完成 text-match / DOM-visible `wait_for`、session 累積的 network / console
viewer、`fill_form` batch sugar，以及 bounded history stack 的 `navigate_back`。
`alert` / `confirm` / `prompt` 會在 DOM runtime 中攔截並累積供 session viewer
讀取；happy-dom 沒有真實 modal UI，因此 `confirm` 回傳 `true`，`prompt` 回傳
呼叫端提供的預設值。真實 blocking dialog 行為仍屬 Phase 4 browser escalation。

**驗證 gate：**同一 session 可完成 navigate、snapshot、act、fill、wait，並讀取
累積的 network / console / dialog 結果；MCP descriptors、嚴格 `fill_form` 轉換與
runtime diagnostics 均有 focused tests。

### Phase 3 — Multi-tab（未完成）

session 需要持有 page map 與 active document，並確保不同 tab 的 JS state 隔離。
這是 session core 的架構變更，優先級低於 Phase 2。

**驗證 gate：**兩個 tab 各自維持獨立 JS state，切換後 snapshot 與 action 都只
作用於 active tab。

### Phase 4 — Real browser escalation（延後）

Screenshot、真實 layout、resize、overlay hit-testing、OS-level file upload 與
真實 DataTransfer DnD 不由 happy-dom 假造。現階段保留外部真實瀏覽器工具作為
互補方案；待純 Rust browser engine 成熟後，再評估 opt-in integration。

### Phase 5 — Extensible Plugin System（v0.24.0，提案）

Roadmap 文件只記錄方向，實作必須等本文件與 plugin spec 對齊後才開始。
目前狀態是 **proposal / approved for planning**，不是 implemented：

- 中立的 dependency-light plugin API 與 compile-time registration。
- `pre_request`、`post_response`、`on_dehydrate` lifecycle hooks。
- core 保留 deadline、cancellation、retry budget、validation 與 MCP error
  conversion；plugin 只能提出 bounded action，不能接管無限控制流。
- 不做 `.so` / `.dylib` Rust trait-object loading；穩定 ABI、version negotiation
  與 crash isolation 另行設計。
- public repository 只提供框架與 generic contract，不放任何外部實作、憑證或
  plugin-specific code。

`spec/plugin-system/spec.md` 是 v0.24.0 的準則；先完成 roadmap 文件校正，
再進入 trait / registry / pipeline integration 的實作規劃。

## 長期原則

- DOM-only 能力留在 in-process V8 + happy-dom，避免不必要的 browser dependency。
- 真實瀏覽器是 escalation，不是核心預設路徑。
- fork release version 遵守「upstream 最新 minor + 1」，只在自己的 milestone
  發布，不追著 upstream 每個版本同步發行。
- 吸納外部設計時記錄來源與邊界；upstream sync 時明確標記 absorb、divert 或
  defer，避免把規劃、外部功能與已完成實作混為一談。
- 公開文件不包含本機絕對路徑、私有環境拓撲、憑證、靜態私人資料或未公開
  外掛細節。

## `[待討論]`

- 靜態 scrape path 是否需要 accessibility output format。
- Remote MCP transport 的完整 session / SSE contract 與部署優先級。
- Plugin cache interception 是否需要獨立 lifecycle，而不是塞進三個既有 hooks。
- Real-browser escalation 的純 Rust engine 與 opt-in integration 時機。
