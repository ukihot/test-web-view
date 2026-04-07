# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.9.x   | :white_check_mark: |
| < 0.9   | :x:                |

## Reporting a Vulnerability

[GitHub Security Advisories](https://github.com/ukihot/test-web-view/security/advisories/new) から非公開で報告してください。

## Updater 署名鍵の管理

Tauri Updater は Ed25519 鍵ペアでアーティファクトの真正性を検証する。秘密鍵が漏洩した場合、または定期ローテーションのために鍵を更新する手順を以下に示す。

### 鍵ペアの生成

```bash
cargo tauri signer generate -w ~/.tauri/vim-browser.key
```

- 秘密鍵: `~/.tauri/vim-browser.key`（**絶対にコミットしない**）
- 公開鍵: ターミナルに出力される文字列

パスワードの設定は任意。設定した場合は CI にも `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` を追加する。

### 鍵の更新手順

1. **新しい鍵ペアを生成する**

   ```bash
   cargo tauri signer generate -w ~/.tauri/vim-browser.key.new
   ```

2. **`tauri.conf.json` の公開鍵を差し替える**

   ```jsonc
   {
     "plugins": {
       "updater": {
         "pubkey": "<新しい公開鍵の文字列>"
       }
     }
   }
   ```

3. **GitHub Secrets を更新する**

   - `TAURI_SIGNING_PRIVATE_KEY` → 新しい秘密鍵の中身
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` → 新しいパスワード（設定した場合）

4. **新バージョンをリリースする**

   鍵更新後の最初のリリースで `latest.json` と `.sig` が新しい鍵で署名される。

5. **旧鍵を破棄する**

   ```bash
   rm ~/.tauri/vim-browser.key
   mv ~/.tauri/vim-browser.key.new ~/.tauri/vim-browser.key
   ```

### 注意事項

- **旧バージョンのアプリ** は旧公開鍵をバイナリに埋め込んでいるため、鍵更新後のリリースを検証できずアップデートに失敗する。ユーザーは手動で新バージョンをインストールする必要がある。
- 秘密鍵は `.gitignore` で除外済み（`.tauri/`）。万が一コミットした場合は即座に鍵を再生成し、`git filter-repo` 等で履歴から除去すること。
- CI では秘密鍵を GitHub Secrets (`TAURI_SIGNING_PRIVATE_KEY`) 経由でのみ参照する。リポジトリのログやアーティファクトに秘密鍵が露出していないか確認すること。
