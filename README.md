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
- `ui` webview: 下端36pxの透過オーバーレイ、ステータスラインと入力欄

## 操作

モードは **NORMAL** と **COMMAND** の2つ。ステータスラインの左端に表示される。

| キー | モード | 動作 |
|------|--------|------|
| `Esc` / `jj` (210ms以内) | 両方 | モード切替 |
| `:` | COMMAND | URL入力モードを開く |
| `h` | COMMAND | 前のバッファへ |
| `l` | COMMAND | 次のバッファへ |
| `Ctrl+w` | 両方 | 現在のバッファを閉じる |
| `Enter` | 入力中 | 現在のバッファでページ遷移 |
| `Ctrl+Enter` | 入力中 | 新規バッファを追加してページ遷移 |
| `Esc` | 入力中 | 入力モードを閉じる |

バッファ一覧はステータスライン右端に `1:host 2:host` 形式で表示される。

起動時には `about:blank` のバッファが1つ用意される。最後の1バッファを閉じた場合も空にはならず、`about:blank` に戻る。

## エンジンアクティビティリール

ステータスラインの中央セクションに、ブラウザエンジンの裏側の動きをリアルタイム表示するアクティビティリールがある。

### キャプチャ対象

`ACTIVITY_INIT_SCRIPT` (browser webviewの初期化スクリプト) が以下のAPIをフックし、通信・状態変化をキャプチャする:

| 種別 | フック方法 | 内容 |
|------|-----------|------|
| `ws.*` | `WebSocket` コンストラクタ差し替え | 接続/送信/受信/切断 |
| `fetch.*` | `window.fetch` ラップ | リクエスト/レスポンス/エラー |
| `xhr.*` | `XMLHttpRequest.prototype` パッチ | open/send/loadend |
| `beacon` | `navigator.sendBeacon` ラップ | ビーコン送信 |
| `sse.*` | `EventSource` コンストラクタ差し替え | SSE接続/メッセージ/エラー |
| `sw.*` | `navigator.serviceWorker` 監視 | SW登録/コントローラー変更 |
| `net.*` | `PerformanceObserver` (resource) | リアルタイムリソース読込 |
| `store.*` | `Storage.prototype.setItem` パッチ | localStorage書き込み |
| `cookie.*` | `CookieStore` change イベント | Cookie変更/削除 |

Tauri IPC通信 (`ipc.localhost`, `tauri.localhost`, `__TAURI_IPC__`) は自動除外され、フィードバックループを防止する。

### リールアニメーション設計

CSS transitionではなく `requestAnimationFrame` + lerp（線形補間）によるフレーム駆動アニメーションを採用している。

```
target (目標位置)
  │
  │  diff = target - current
  │  current += diff * 0.09   ← 毎フレーム9%だけ近づく
  │
current (現在位置) ──→ 自然な減速カーブで滑らかに収束
```

**なぜCSS transitionを使わないか:**

ネットワークアクティビティは不定期かつ高頻度で発生する。CSS transitionだと:
1. transition中に新しいエントリが来るとtargetが変わり、中間位置でカクつく
2. バッチ化しても「バッチ間の停止」が発生し、流れが途切れる
3. `transitionend` の管理が複雑になる

rAF + lerpなら:
1. いつ何個エントリが来ても `target` を更新するだけ — 現在進行中の動きに自然に合流
2. 行間で停止しない — 常に目標に向かって動き続ける
3. 状態管理がシンプル — `current` と `target` の2変数のみ

**ドラムリール3D効果:**

各行にビューポート内の相対位置に応じた `rotateX` / `translateZ` / `opacity` を毎フレーム適用:

```
行の位置          rotateX    translateZ    opacity
──────────────────────────────────────────────────
上端 (dist=-1.5)   +30°       -2px         0.0
中央 (dist= 0.0)    0°        0px          1.0
下端 (dist=+1.5)   -30°       -2px         0.0
```

`perspective(120px)` を各行に個別指定し、コンテナの `translateY` と干渉しない構成。上下端のCSSグラデーションフェードと組み合わせて、覗き窓越しにドラムが回転する見た目を実現。

**GC（ガベージコレクション）:**

DOMが際限なく膨らむのを防ぐため、21行を超えた古い行を削除する。削除分だけ `current` / `target` を同量引くことで、視覚的なジャンプなしにメモリを回収する。

### 認証トークン検出

ページロード完了時に cookie / localStorage / sessionStorage をスキャンし、`token`, `session`, `auth`, `jwt`, `sid`, `csrf` 等のパターンに一致するキーがあればステータスライン左側に 🔑 バッジで表示する。

## 開発

```bash
cargo tauri dev
```

## ビルド

```bash
cargo tauri build
```

