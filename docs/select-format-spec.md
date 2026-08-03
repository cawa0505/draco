# Select format spec — CSS selector extraction

Status: **implemented (v1)** · Area: wire contract (draco-types), core (FormatSet/Config/machine), static (selector engine), CLI, daemon, MCP

## 1. Motivation

ax-scraper 的 CSS selector 萃取（`scrape_web(url, selector)` → 匹配節點文字）是 Draco 取代它時唯一缺的能力。既有 `--extract`（`extract_schema.rs`）是 schema 驅動的結構化萃取——要寫 JSON schema；這裡補一個裸 selector 模式：給一個 CSS selector，拿回匹配節點的 text + HTML。

## 2. Design

**新 format `select`** + **重複 flag `--selector <CSS>`**（`Config.selectors: Vec<String>`）。

- `--selector` 隱含 `select` format（`--format select` 可不寫）。
- `--format select` 但沒給 `--selector` → reject（CLI exit 1 / daemon 400），與 `parse_formats` 的既有 reject 同級。
- 無匹配 → `matches: []`（非錯誤）。
- 非法 selector → 請求層 reject（`DracoError::Config`），不像 `extract_schema` 那樣 warn-and-null——這是使用者直接打的 flag，打錯要響。

**輸出契約**（`ExtractionResult` 加一個 additive 欄位，與 `links`/`endpoints` 同級，`None` 時從 wire 省略）：

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

- 每個輸入 selector 一組，全匹配，上限沿用 `extract_schema::MAX_MATCHES`（1000）。
- `text`：whitespace 收斂的文字內容（複用 `extract_schema::collapse_ws`）。
- `html`：匹配元素的 **outerHTML**，raw（不做 absolutize/清洗——那是 markdown/html 管線的事，這裡要忠實原樣）。

## 3. Staging — select 完全鏡像 `html` format

- 靜態階段：與 `html` 同一個 DOM（`filter_body` 之後、`machine.rs` `static` 分支），與 include/exclude-tags 自然組合。
- render 升級：hydrated DOM 刷新 `html` 的同一個點（`machine.rs` ~1266）同步刷新 select——SPA 也能選。
- `FormatSet::select` 併入 `wants_static_content()`（`is_static_terminal` 自動涵蓋）。
- trace：`static.select` / 刷新時 `runtime.render` 之後的 select step，記錄匹配數。
- 各 surface 的輸出 gating 比照 `html` 現行行為。

## 4. Surfaces

| Surface | v1 | 註 |
|---|---|---|
| CLI `draco scrape` | ✅ `--format select --selector "…"`（repeatable） | |
| daemon `POST /v1/scrape` | ✅ `formats: ["select"]` + `selector`（string 或 array） | Draco extension 欄位，同 `tierMax` 定位；`data.selector` 進 envelope |
| MCP `draco_scrape` | ✅ `selector` param（string 或 array） | schema enum 加 `"select"` |
| crawl / batch / search | [待討論] | formats 已走 `parse_formats`，但 selector 需穿過 `PageQuery`；v1 不穿 |
| interact scrape | [待討論] | live DOM 也能選，但 scope 先不收 |

## 5. Files touched（blast radius 8）

1. `crates/draco-types/src/lib.rs` — `ExtractionResult.selector: Option<Vec<SelectorMatch>>` + `SelectorMatch{selector,text,html}`；roundtrip/omission 測試補欄位
2. `crates/draco-core/src/lib.rs` — `FormatSet.select` + `Config.selectors`
3. `crates/draco-core/src/machine.rs` — 兩個 staging 點 + trace step
4. `crates/draco-core/src/interact.rs` — `ExtractionResult` 建構點補 `selector: None`
5. `crates/draco-static/src/extract_schema.rs` — `select_matches(html, selectors) -> (Vec<SelectorMatch>, Vec<String>)`，複用 `collapse_ws`/`MAX_MATCHES` + 單元測試
6. `crates/draco-cli/src/main.rs` — `--selector` flag + `--format select` 解析
7. `crates/draco-cli/src/serve/mod.rs` — `parse_formats` 收 `select` + `to_firecrawl` 映射 + `/v1/scrape` request 欄位
8. `crates/draco-cli/src/mcp.rs` — tool schema + `selector` param

測試：draco-static `select_matches` 單元測試（text 收斂、outerHTML、無匹配、cap）、types roundtrip、serve `parse_formats`。

## 6. 不做（v1）

- 多 selector 合併輸出（各自分組即可）
- `text`/`html` 以外的序列化（markdown of match → 可用 `--include-tag` + markdown 達成）
- crawl/batch/search 穿 selector（[待討論]）

## 7. 驗證

```
cargo test -p draco-static -p draco-types
cargo test -p draco-cli -- serve::tests
draco scrape https://example.com --format select --selector "h1"
```
