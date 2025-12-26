# Auto Video Organize

### 概览
此仓库聚焦于一致的发布流程、可靠的自动化与清晰的项目文档。README 保持高层描述，不涉及实现细节。

### 功能
本工具提供了一套实用程序来管理大型视频收藏：

- **视频转码器 (Video Encoder)**：使用 FFmpeg 批量将视频转换为 HEVC/x265 格式，在保持画质的同时节省空间。
- **重复文件检查器 (Duplication Checker)**：使用 BLAKE3 哈希算法识别重复文件，并将它们移动到独立目录以清理存储空间。
- **缩略图目录生成器 (Contact Sheet Generator)**：自动为视频生成预览缩略图目录。利用场景检测选取具代表性的时间点，并并行处理缩略图以提升速度。
- **按类型自动移动 (Auto Move by Type)**：扫描目录并根据扩展名将文件整理到子文件夹中。
- **孤立文件移动器 (Orphan File Mover)**：检测并重新安置“孤立”文件——例如不再有对应视频文件的附属文件或缩略图。
- **视频重命名器 (Video Renamer)**：依据视频时长排序并重命名文件，以保持特定的顺序。

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
