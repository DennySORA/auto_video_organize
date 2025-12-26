# Auto Video Organize

## English

### Overview
This repository focuses on consistent releases, reliable automation, and clear project documentation. The README stays intentionally high-level and avoids implementation details.

### Install
One-line install (builds from source):
```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Auto-Video-Organize/main/install.sh | bash
```

Requirements:
- `git`
- Rust toolchain (`cargo`)

Release binaries (recommended):
1. Download the asset matching your OS/arch from GitHub Releases.
2. Extract it (`.tar.gz` on macOS/Linux, `.zip` on Windows).
3. Move the binary to a directory on your `PATH` (e.g. `~/.local/bin`).

Source build (manual):
1. `git clone https://github.com/DennySORA/Auto-Video-Organize.git`
2. `cd Auto-Video-Organize`
3. `cargo build --release --locked`
4. Copy `target/release/auto_video_organize` to a directory on your `PATH`.

Script options:
- `REF` to select a tag/branch (default: `main`).
- `PREFIX` or `BIN_DIR` to change the install location.

### CI/CD
- Security scanning for dependency advisories
- Unit tests on every push and pull request
- Automatic Release creation when a tag is pushed (e.g. v1.2.3)
- Release assets include prebuilt binaries

### Contributing
Open an issue or pull request with a clear scope and rationale.

### Support
Use GitHub Issues for questions and requests.

---

## 繁體中文

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

---

## 简体中文

### 概览
此仓库聚焦于一致的发布流程、可靠的自动化与清晰的项目文档。README 保持高层描述，不涉及实现细节。

### 安装
一行安装（从源码构建）：
```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Auto-Video-Organize/main/install.sh | bash
```

需求：
- `git`
- Rust 工具链（`cargo`）

Release 二进制（推荐）：
1. 到 GitHub Releases 下载对应操作系统/架构的文件。
2. 解压（macOS/Linux 为 `.tar.gz`，Windows 为 `.zip`）。
3. 将可执行文件移动到 `PATH` 目录（例如 `~/.local/bin`）。

源码构建（手动）：
1. `git clone https://github.com/DennySORA/Auto-Video-Organize.git`
2. `cd Auto-Video-Organize`
3. `cargo build --release --locked`
4. 将 `target/release/auto_video_organize` 复制到 `PATH` 目录。

脚本选项：
- `REF` 指定 tag/branch（默认：`main`）。
- `PREFIX` 或 `BIN_DIR` 调整安装路径。

### CI/CD
- 安全扫描（依赖项漏洞通告）
- 每次 push 与 PR 执行单元测试
- 打 tag 时自动发布 Release（例如 v1.2.3）
- Release 内含预编译二进制

### 贡献
欢迎提交 issue 或 PR，请清楚说明范围与理由。

### 支持
问题与需求请使用 GitHub Issues。

---

## 日本語

### 概要
このリポジトリは、一貫したリリース、信頼できる自動化、明確なドキュメントを重視しています。README は高レベルに留め、実装の詳細には触れません。

### インストール
ワンライン（ソースからビルド）：
```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Auto-Video-Organize/main/install.sh | bash
```

要件：
- `git`
- Rust ツールチェーン（`cargo`）

Release バイナリ（推奨）：
1. GitHub Releases から OS/アーキテクチャに合うアセットを取得します。
2. 解凍します（macOS/Linux は `.tar.gz`、Windows は `.zip`）。
3. 実行ファイルを `PATH` 配下（例: `~/.local/bin`）へ移動します。

ソースビルド（手動）：
1. `git clone https://github.com/DennySORA/Auto-Video-Organize.git`
2. `cd Auto-Video-Organize`
3. `cargo build --release --locked`
4. `target/release/auto_video_organize` を `PATH` 配下へコピーします。

スクリプトのオプション：
- `REF` で tag/branch を指定（デフォルト: `main`）。
- `PREFIX` または `BIN_DIR` でインストール先を変更。

### CI/CD
- 依存関係の脆弱性チェック
- プッシュ／PR ごとのユニットテスト
- タグ作成時に自動で Release を公開（例: v1.2.3）
- Release にプリビルドのバイナリを同梱

### コントリビュート
Issue または PR を歓迎します。範囲と理由を明確にしてください。

### サポート
質問や要望は GitHub Issues を利用してください。
