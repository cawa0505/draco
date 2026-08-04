# Draco Feature Spec: Stealth Proxy & Adaptive Rate Limit Engine

*   **Feature Name:** Stealth Proxy & Adaptive Rate Limit Engine (抗封鎖代理與自適應限流引擎)
*   **Target Core:** `draco-core` / `draco-mcp` / `draco-net`
*   **Version:** v0.23.0
*   **Status:** `in-progress`

## 1. 背景與痛點 (Background & Problem)
在實測電商（如 momo）等高強度防爬網站時，雖然 draco 具備極致的 Token 脫水與微秒級解析能力，但連爬時容易觸發 WAF 的 429 (Too Many Requests) 與 403 (IP/ASN Block)。
- 痛點 A：機房 IP (Datacenter IP) 易被一刀切。
- 痛點 B：固定頻率請求易被 WAF 演算法識別為機器人。
- 痛點 C：被限流時需要 Agent 手動重試，浪費 Token 與 Context 週期。

## 2. 核心功能目標 (Core Goals)
- **Transparent Proxy Rotation (透明代理輪換)**：支援 SOCKS5/HTTP 代理池（含 Residential Proxy 與 Tor 輪換），被封時 Rust 底層自動切換，對上層 MCP 零感。
- **Humanized Jitter Delay (人性化隨機延遲)**：內建隨機抖動延遲，擺脫固定週期請求特徵。
- **Browser Header Emulation (全套瀏覽器 Header 擬真)**：自動補齊 `Sec-Ch-Ua`、`Referer` 等 Header 以提供連貫的瀏覽器指紋特徵。
- **Auto-Retry & Circuit Breaker (自動降級熔斷)**：收到 429/403 時自動進行指數退避（Exponential Backoff）與 IP 輪換重試。

## 3. 設定檔規格 (draco.toml / [stealth])
在設定檔中新增防護配置。如果設定檔不存在，則預設不啟用或啟用全域預設：
```toml
[stealth]
enabled = true
user_agent_mode = "desktop_chrome_random"
default_jitter_ms = [800, 2500]

[stealth.headers]
referer_emulation = true
sec_ch_ua_auto = true

[proxy]
enabled = true
mode = "rotate_on_blocked"
max_retries = 3
endpoints = [
    "socks5://127.0.0.1:9050",
    "http://127.0.0.1:8888"
]

[domains."momoshop.com.tw"]
jitter_ms = [2000, 4500]
proxy_mode = "always_rotate"
```

## 4. 驗證與測試設計 (Validation)
- 單元測試與整合測試。
- 驗證 Header 注入、隨機延遲以及 429/403 時的 Proxy 自動輪換機制。
- `[待討論]`：是否需要支援 `on_block_cmd` 觸發外部腳本？本版優先實作純 Rust 的代理輪換與自動退避，外部腳本呼叫暫列為可選。
