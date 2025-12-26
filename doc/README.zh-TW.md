# Auto Video Organize

### 概覽
此倉庫以一致的發佈流程、可靠的自動化與清楚的專案文件為目標，README 刻意保持高層描述，不涉及實作細節。

### 安裝
一行安裝（從原始碼建置）：
```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Auto-Video-Organize/main/install.sh | bash
```

需求：
- `git`
- Rust 工具鏈（`cargo`）

Release 二進位（建議）：
1. 到 GitHub Releases 下載對應作業系統/架構的檔案。
2. 解壓縮（macOS/Linux 為 `.tar.gz`，Windows 為 `.zip`）。
3. 將執行檔移到 `PATH` 目錄（例如 `~/.local/bin`）。

原始碼建置（手動）：
1. `git clone https://github.com/DennySORA/Auto-Video-Organize.git`
2. `cd Auto-Video-Organize`
3. `cargo build --release --locked`
4. 將 `target/release/auto_video_organize` 複製到 `PATH` 目錄。

腳本選項：
- `REF` 指定 tag/branch（預設：`main`）。
- `PREFIX` 或 `BIN_DIR` 調整安裝路徑。

### CI/CD
- 安全掃描（相依性漏洞通報）
- 每次 push 與 PR 執行單元測試
- 以 tag 觸發自動發佈 Release（例如 v1.2.3）
- Release 內含預先編譯的二進位

### 貢獻
歡迎提交 issue 或 PR，請清楚描述範圍與理由。

### 支援
問題與需求請使用 GitHub Issues。
