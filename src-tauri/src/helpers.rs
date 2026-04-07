use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    constants::{BROWSER_LABEL, UI_LABEL},
    domain::{Mode, Snapshot},
};

pub fn normalize_url(raw: &str) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_owned()
    } else {
        format!("https://{raw}")
    }
}

pub fn emit_snapshot(app: &AppHandle, snap: &Snapshot) {
    emit_to_ui(app, "state-change", snap);
}

pub fn emit_to_ui<S: Serialize + Clone>(app: &AppHandle, event: &str, payload: &S) {
    if let Some(ui) = app.get_webview(UI_LABEL) {
        let _ = ui.emit(event, payload.clone());
    }
}

pub fn get_browser(app: &AppHandle) -> Result<tauri::Webview, String> {
    app.get_webview(BROWSER_LABEL)
        .ok_or_else(|| "browser webview not found".to_owned())
}

pub fn navigate_browser(app: &AppHandle, url: &str) -> Result<(), String> {
    let parsed: url::Url = url.parse().map_err(|e: url::ParseError| e.to_string())?;
    get_browser(app)?
        .navigate(parsed)
        .map_err(|e| e.to_string())
}

pub fn set_focus_for_mode(app: &AppHandle, mode: Mode) {
    match mode {
        Mode::Command => {
            if let Some(wv) = app.get_webview(UI_LABEL) {
                let _ = wv.set_focus();
            }
        }
        Mode::Normal => {
            if let Some(browser) = app.get_webview(BROWSER_LABEL) {
                let _ = browser.set_focus();
                // Reset the IPC flag before probing.
                if let Ok(mut guard) = app.state::<crate::state::ManagedState>().lock_or_err() {
                    guard.browser_ipc_ok = false;
                }
                // On error pages the Tauri IPC bridge is unavailable.
                // Schedule a ping probe: if browser doesn't respond within
                // 200ms, fall back to UI which always has a keydown handler.
                let _ = browser.eval(
                    "(function(){try{if(window.__TAURI__&&window.__TAURI__.core){window.__TAURI__.core.invoke('browser_ping').catch(function(){})}}catch(_){}})()",
                );
                let handle = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    let managed = handle.state::<crate::state::ManagedState>();
                    let st = managed.lock_or_err();
                    if let Ok(guard) = st
                        && guard.browser_ipc_ok
                    {
                        return; // ping arrived, browser has IPC — keep focus
                    }
                    // No ping received — fall back to UI
                    if let Some(ui) = handle.get_webview(UI_LABEL) {
                        let _ = ui.set_focus();
                    }
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_url_with_scheme_unchanged() {
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
        assert_eq!(normalize_url("http://example.com"), "http://example.com");
    }

    #[test]
    fn normalize_url_without_scheme_adds_https() {
        assert_eq!(normalize_url("example.com"), "https://example.com");
        assert_eq!(
            normalize_url("rust-lang.org/learn"),
            "https://rust-lang.org/learn"
        );
    }

    #[test]
    fn normalize_url_empty_string() {
        assert_eq!(normalize_url(""), "https://");
    }
}
