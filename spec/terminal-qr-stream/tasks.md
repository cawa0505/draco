# Terminal ANSI QR Stream & Headless Auth Relay — Tasks

- [ ] **1. Decoder (draco-core::qr)**
  - [ ] 1.1 實作 DOM Canvas / Base64 Image / Auth URL 到 QR payload 的解碼器
  - [ ] 1.2 單元測試：Canvas 與不同 Base64 圖像尺寸解析
- [ ] **2. Renderer (draco-core::qr)**
  - [ ] 2.1 建立 `TerminalQrRenderer`，支援 ANSI 顏色翻轉 (`invert_colors`)
  - [ ] 2.2 支援 Unicode 1x2 緊湊壓縮字元渲染 (`compact`)
  - [ ] 2.3 支援 OSC 8 超連結 fallback 輸出
- [ ] **3. CLI & MCP 整合**
  - [ ] 3.1 CLI 新增 `draco auth` 子指令
  - [ ] 3.2 整合 `draco-mcp`：遭遇 401 登入時發送 Stream 通知提請使用者掃碼驗證
