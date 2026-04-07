use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

/// バックグラウンドでアップデートチェックを行う。
pub fn spawn_update_check(handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let updater = match handle.updater() {
            Ok(u) => u,
            Err(e) => {
                eprintln!("[updater] init failed: {e}");
                return;
            }
        };
        match updater.check().await {
            Ok(Some(update)) => {
                eprintln!(
                    "[updater] new version available: {} (current: {})",
                    update.version, update.current_version
                );
                if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
                    eprintln!("[updater] install failed: {e}");
                }
            }
            Ok(None) => {
                eprintln!("[updater] up to date");
            }
            Err(e) => {
                eprintln!("[updater] check failed: {e}");
            }
        }
    });
}
