# Terminal ANSI QR Stream & Headless Auth Relay

Status: `draft`  
Target: `draco-core` / `draco-cli` / `draco-mcp`  
Version: v0.27.0  
Prerequisites: v0.25.0-plugin-system completed

## 1. 概述 (Context & Problem)

在自動化網頁操控與 DOM 脫水過程中，許多現代平台（如微信網頁版、LINE 後台、淘寶/蝦皮賣家中心、Google Cloud Console 授權）會強制跳出 QR Code 掃描登入/二重驗證。

傳統無頭瀏覽器（Headless Browser）在此情境下通常需要掛載 VNC、打開重量級 GUI 視窗，或是將二維碼存為本地圖片檔案再由人工打開，中斷自動化管線。

本規格定義 「Terminal ANSI QR Stream」 引擎：
- **DOM QR 圖像/網址自動捕捉**：微秒級識別網頁 DOM 中的 `<canvas>`、Base64 `<img>` 或 Auth URL。
- **Zero-GUI Terminal 繪製**：透過 ANSI Unicode (Dense Block 1x2) 高密度雙色渲染，直接在 CLI/Terminal 輸出可被手機鏡頭識別的二維碼。
- **Session 自動接管 (Session Handshake)**：掃碼完成後，draco 背景 Task 無縫捕捉 Cookie / Storage，立刻恢復 DOM 脫水與數據擷取任務。

## 2. 系統架構與互動流程 (Architecture Sequence)

```text
┌────────────────┐      ┌────────────────┐      ┌────────────────┐      ┌────────────────┐
│ Target Web WAF │      │  draco-core    │      │ User Terminal  │      │  Mobile Phone  │
│ (e.g. Line/GCP)│      │  (Rust Engine) │      │  (CLI / TUI)   │      │ (Camera App)   │
└───────┬────────┘      └───────┬────────┘      └───────┬────────┘      └───────┬────────┘
        │                       │                       │                       │
        │ 1. Nav to Auth Page   │                       │                       │
        │◄──────────────────────┤                       │                       │
        │                       │                       │                       │
        │ 2. Return QR Element  │                       │                       │
        ├──────────────────────►│                       │                       │
        │                       │                       │                       │
        │                       │ 3. Parse Canvas/Base64│                       │
        │                       │    & Render ANSI QR   │                       │
        │                       │──────────────────────►│                       │
        │                       │                       │ 4. Scan QR Code       │
        │ 5. Auth Success       │                       │◄──────────────────────┤
        │◄──────────────────────────────────────────────┴───────────────────────┤
        │                       │                       │                       │
        │ 6. Pass Session Cookie│                       │                       │
        ├──────────────────────►│                       │                       │
        │                       │ 7. [✔] Auth Captured! │                       │
        │                       │    Resume Crawling... │                       │
        │                       │──────────────────────►│                       │
```

## 3. 核心 API 與 CLI 介面設計

### A. CLI Command
```bash
# 啟動微型登入捕捉器並在 Terminal 輸出 QR Code
draco auth --url "https://target-site.com/login" --qr-selector "#qr-canvas"

# 輸出效果：
# ┌────────────────────────────────────────────────────────┐
# │                                                        │
# │   [draco] QR Code Auth Required                        │
# │                                                        │
# │   ▄▄▄▄▄▄▄  ▄  ▄▄ ▄▄▄▄▄▄▄                               │
# │   █ ▄▄▄ █  ▀  ▀█ █ ▄▄▄ █    Scan with your mobile      │
# │   █ ███ █ ▀▀▀█ █ █ ███ █    camera to authorize.       │
# │   █▄▄▄▄▄█ █▀█▀█▀ █▄▄▄▄▄█                               │
# │   █ ▄▄ ▄▄▀  ▀█ ▀█▄  ▄  █    [Target]: LINE Seller      │
# │   █ █  ▄▀▀  ▀█▀▀▀█ ▀▄▀ █    [TTL]: 120s                │
# │   ▄▄▄▄▄▄▄ █▀▄█ ▀  ▀  ▀ █                               │
# │   █ ▄▄▄ █ ▄▀  ▀▀ █▀█ ▀ █                               │
# │   █ ███ █ █▀ █▄ ▀█ ▄ ▀ █                               │
# │   █▄▄▄▄▄█ █ ▀  ▀█  ▀ ▀ █                               │
# │                                                        │
# │   Listening for Session Handshake... [30s]             │
# └────────────────────────────────────────────────────────┘
```

### B. Rust API (`draco-core::qr`)
```rust
use draco_core::qr::{TerminalQrRenderer, QrSource};

pub async fn handle_qr_login(page: &mut DracoPage) -> Result<Session, DracoError> {
    // 1. 自動偵測 DOM 中的 Canvas 或 Base64 圖像
    let qr_element = page.find_qr_element("#login-qrcode").await?;
    
    // 2. 解碼成 ANSI 字串
    let ansi_qr = TerminalQrRenderer::new()
        .invert_colors(true) // 適應黑色 Terminal 背景
        .compact(true)       // 使用 1x2 區塊字元縮小體積 50%
        .render_from_element(&qr_element)?;

    // 3. 印出 Terminal
    println!("{}", ansi_qr);

    // 4. 等待 Auth Session 轉向完成
    let session = page.wait_for_session_cookie("session_id", Duration::from_secs(120)).await?;
    Ok(session)
}
```

## 4. 關鍵設計細節 (Key Design Considerations)

- **Terminal 色彩反轉 (Dark Mode Inversion)**：多數工程師的 Terminal 為黑底。傳統 QR Code 為白底黑字，若未反轉顏色的 Dark/Light 屬性，手機鏡頭會因為對比反相而無法識別。`TerminalQrRenderer` 預設啟用 `invert_colors = true`。
- **高密度 1x2 區塊壓縮 (Compact Density)**：使用 Unicode Dense 1x2 字元（`▀`、`▄`、`█`），在單一字元空間內繪製上下兩像素點，使二維碼體積在 Terminal 中縮小 50%，避免因視窗寬度不足導致換行破版。
- **網址 Fallback 與超連結 (OSC 8 Hyperlink)**：若 Terminal 支援，同時輸出 OSC 8 隱形超連結與短網址，讓使用者即使無法用手機掃碼，也能直接點擊網址打開瀏覽器。
