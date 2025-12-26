# Auto Video Organize

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
