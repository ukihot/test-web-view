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
    let label = match mode {
        Mode::Command => UI_LABEL,
        Mode::Normal => BROWSER_LABEL,
    };
    if let Some(wv) = app.get_webview(label) {
        let _ = wv.set_focus();
    }
}
