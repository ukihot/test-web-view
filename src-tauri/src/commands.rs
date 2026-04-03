use tauri::{AppHandle, State};

use crate::{
    domain::{Mode, ResourceEntry, Snapshot},
    helpers::{
        emit_snapshot, emit_to_ui, get_browser, navigate_browser, normalize_url, set_focus_for_mode,
    },
    state::ManagedState,
};

#[tauri::command]
pub fn get_state(state: State<'_, ManagedState>) -> Result<Snapshot, String> {
    Ok(state.lock_or_err()?.snapshot())
}

#[tauri::command]
pub fn toggle_mode(app: AppHandle, state: State<'_, ManagedState>) -> Result<Snapshot, String> {
    let snap = state.lock_or_err()?.toggle_mode();
    emit_snapshot(&app, &snap);
    set_focus_for_mode(&app, snap.mode);
    Ok(snap)
}

#[tauri::command]
pub fn navigate_to(
    app: AppHandle,
    state: State<'_, ManagedState>,
    url: String,
    new_buffer: Option<bool>,
) -> Result<(), String> {
    let normalized = normalize_url(&url);
    let parsed: url::Url = normalized
        .parse()
        .map_err(|e: url::ParseError| e.to_string())?;

    let mut st = state.lock_or_err()?;
    let snap = if new_buffer.unwrap_or(false) {
        st.add_buffer(normalized.clone())
    } else {
        st.navigate_active(normalized.clone())
    };
    emit_snapshot(&app, &snap);
    emit_to_ui(&app, "page-load-start", &normalized);

    get_browser(&app)?
        .navigate(parsed)
        .map_err(|e| e.to_string())?;

    set_focus_for_mode(&app, Mode::Normal);
    Ok(())
}

#[tauri::command]
pub fn buffer_next(app: AppHandle, state: State<'_, ManagedState>) -> Result<(), String> {
    let (snap, nav_url) = state.lock_or_err()?.cycle_buffer(1).ok_or("no buffers")?;
    emit_snapshot(&app, &snap);
    navigate_browser(&app, &nav_url)
}

#[tauri::command]
pub fn buffer_prev(app: AppHandle, state: State<'_, ManagedState>) -> Result<(), String> {
    let (snap, nav_url) = state.lock_or_err()?.cycle_buffer(-1).ok_or("no buffers")?;
    emit_snapshot(&app, &snap);
    navigate_browser(&app, &nav_url)
}

#[tauri::command]
pub fn close_current_buffer(app: AppHandle, state: State<'_, ManagedState>) -> Result<(), String> {
    let (snap, nav_url) = state.lock_or_err()?.close_active_buffer();
    emit_snapshot(&app, &snap);
    emit_to_ui(&app, "page-load-start", &nav_url);
    navigate_browser(&app, &nav_url)
}

#[tauri::command]
pub fn report_title(
    app: AppHandle,
    state: State<'_, ManagedState>,
    title: String,
) -> Result<(), String> {
    let snap = state.lock_or_err()?.set_active_title(title);
    emit_snapshot(&app, &snap);
    Ok(())
}

#[tauri::command]
pub fn report_resources(app: AppHandle, resources: Vec<ResourceEntry>) -> Result<(), String> {
    emit_to_ui(&app, "resource-log", &resources);
    Ok(())
}
