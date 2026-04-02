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

| キー | 動作 |
|------|------|
| `:` | URL入力モードを開く（アプリ前面時） |
| `Enter` | ナビゲート |
| `Esc` | 入力モードを閉じる |

## 開発

```bash
cargo tauri dev
```

## ビルド

```bash
cargo tauri build
```
