# Auto Video Organize

### 概要
このリポジトリは、一貫したリリース、信頼できる自動化、明確なドキュメントを重視しています。README は高レベルに留め、実装の詳細には触れません。

### 機能
このツールは、大量の動画コレクションを管理するための一連のユーティリティを提供します：

- **動画エンコーダー (Video Encoder)**：FFmpeg を使用して動画を HEVC/x265 形式にバッチ変換し、画質を維持しながら容量を節約します。
- **重複ファイルチェッカー (Duplication Checker)**：BLAKE3 ハッシュを使用して重複ファイルを特定し、別のディレクトリに移動してストレージを整理します。
- **コンタクトシート生成 (Contact Sheet Generator)**：動画のコンタクトシート（サムネイル一覧）画像を自動生成します。シーン検出を使用して意味のあるタイムスタンプを選択し、サムネイルを並列処理して高速化します。
- **タイプ別自動移動 (Auto Move by Type)**：ディレクトリをスキャンし、ファイル拡張子に基づいてサブフォルダに整理します。
- **孤立ファイル移動 (Orphan File Mover)**：対応する動画ファイルが存在しないサイドカーファイルやサムネイルなどの「孤立」ファイルを検出し、移動します。
- **動画リネーム (Video Renamer)**：動画ファイルを再生時間順にソートし、特定の順序を維持するようにリネームします。

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
