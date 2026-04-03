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
| `Ctrl+w` | 両方 | 現在のバッファを閉じる |
| `Enter` | 入力中 | 現在のバッファでページ遷移 |
| `Ctrl+Enter` | 入力中 | 新規バッファを追加してページ遷移 |
| `Esc` | 入力中 | 入力モードを閉じる |

バッファ一覧はステータスライン右端に `1:host 2:host` 形式で表示される。

起動時には `about:blank` のバッファが1つ用意される。最後の1バッファを閉じた場合も空にはならず、`about:blank` に戻る。

## 開発

```bash
cargo tauri dev
```

## ビルド

```bash
cargo tauri build
```

