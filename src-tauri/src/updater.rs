use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

use crate::{helpers::emit_to_ui, state::ManagedState};

#[derive(Serialize, Clone)]
struct UpdateAvailable {
    version: String,
    current: String,
}

/// バックグラウンドでアップデートチェックを行い、
/// 進捗を UI の中央エリアへ通知する。
/// チェック完了後に `update-done` イベントを発火し、
/// アクティビティモニタの稼働を開始させる。
pub fn spawn_update_check(handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        emit_to_ui(&handle, "update-status", &"checking...");

        let updater = match handle.updater() {
            Ok(u) => u,
            Err(e) => {
                eprintln!("[updater] init failed: {e}");
                emit_to_ui(&handle, "update-status", &format!("updater error: {e}"));
                finish(&handle);
                return;
            }
        };

        match updater.check().await {
            Ok(Some(update)) => {
                let info = UpdateAvailable {
                    version: update.version.clone(),
                    current: update.current_version.clone(),
                };
                emit_to_ui(&handle, "update-available", &info);

                // UI 側で Y/N キー入力を受け付け、respond_update コマンドで
                // チャネル経由で応答が届く。
                let (tx, rx) = std::sync::mpsc::channel::<bool>();
                {
                    let managed = handle.state::<ManagedState>();
                    if let Ok(mut guard) = managed.lock_or_err() {
                        guard.update_tx = Some(tx);
                    }
                }

                let accepted =
                    tauri::async_runtime::spawn_blocking(move || rx.recv().unwrap_or(false))
                        .await
                        .unwrap_or(false);

                if !accepted {
                    emit_to_ui(&handle, "update-status", &"update skipped");
                    finish(&handle);
                    return;
                }

                emit_to_ui(&handle, "update-status", &"downloading...");

                match update.download_and_install(|_, _| {}, || {}).await {
                    Ok(_) => {
                        emit_to_ui(&handle, "update-status", &"updated — restart to apply");
                    }
                    Err(e) => {
                        eprintln!("[updater] install failed: {e}");
                        emit_to_ui(&handle, "update-status", &format!("install error: {e}"));
                    }
                }
            }
            Ok(None) => {
                emit_to_ui(&handle, "update-status", &"up to date ✓");
            }
            Err(e) => {
                eprintln!("[updater] check failed: {e}");
                emit_to_ui(&handle, "update-status", &"check error");
            }
        }

        finish(&handle);
    });
}

fn finish(handle: &AppHandle) {
    emit_to_ui(handle, "update-done", &());
}
