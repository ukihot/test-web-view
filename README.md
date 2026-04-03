# vim-browser

Tauri製WebViewラッパー。vimライクなキーバインドでURLナビゲーションができる。

## 構成

```
index.html        # UI（ステータスライン）
styles.css        # スタイル
src-tauri/src/lib.rs  # バックエンド（Tauri）
```

Webviewを2枚重ねる構成:
- `browser` webview: フルスクリーンでWebページを表示
- `ui` webview: 下端22pxの透過オーバーレイ、ステータスラインと入力欄

## 操作

モードは **NORMAL** と **COMMAND** の2つ。ステータスラインの左端に表示される。

| キー | モード | 動作 |
|------|--------|------|
| `Esc` / `jj` (400ms以内) | 両方 | モード切替 |
| `:` | COMMAND | URL入力モードを開く |
| `h` | COMMAND | 前のバッファへ |
| `l` | COMMAND | 次のバッファへ |
| `Enter` | 入力中 | ナビゲート（新規バッファ） |
| `Esc` | 入力中 | 入力モードを閉じる |

バッファ一覧はステータスライン右端に `1:host 2:host` 形式で表示される。

## 開発

```bash
cargo tauri dev
```

## ビルド

```bash
cargo tauri build
```

## GitHubリリース自動化

`.github/workflows/release.yml` で、タグ・リリース・配布物アップロードを自動化している。

- `workflow_dispatch`:
	- `version` (例: `0.1.1`) を入力して実行
	- `v0.1.1` タグを自動作成
	- `cargo tauri build` 実行
	- `src-tauri/target/release/bundle/**` の成果物を GitHub Release に添付
- `push tag`:
	- `v*.*.*` タグを push した場合も同じビルド・公開フローを実行

必要権限:

- `contents: write` (タグ push / Release 作成 / アセット添付)
