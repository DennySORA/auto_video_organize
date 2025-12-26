# Auto Video Organize

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
