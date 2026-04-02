use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl, WindowEvent,
    webview::{PageLoadEvent, WebviewBuilder},
    window::WindowBuilder,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

const BROWSER_LABEL: &str = "browser";
const UI_LABEL: &str = "ui";
const STATUS_H: f64 = 22.0;

#[tauri::command]
async fn navigate_to(app: AppHandle, url: String) -> Result<(), String> {
    let normalized = if url.starts_with("http://") || url.starts_with("https://") {
        url
    } else {
        format!("https://{}", url)
    };
    let parsed: url::Url = normalized.parse().map_err(|e| format!("{e}"))?;

    // ナビゲート前にローディング状態を即座にemit
    if let Some(ui) = app.get_webview(UI_LABEL) {
        let _ = ui.emit("page-load-start", normalized.clone());
    }

    app.get_webview(BROWSER_LABEL)
        .ok_or("browser not found")?
        .navigate(parsed)
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct ResourceEntry {
    name: String,
    duration: f64,
    transfer_size: f64,
    initiator_type: String,
}

#[tauri::command]
async fn report_resources(app: AppHandle, resources: Vec<ResourceEntry>) -> Result<(), String> {
    if let Some(ui) = app.get_webview(UI_LABEL) {
        let _ = ui.emit("resource-log", resources);
    }
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            navigate_to,
            report_resources
        ])
        .setup(|app| {
            let win = WindowBuilder::new(app, "main")
                .title("vim-browser")
                .inner_size(1200.0, 800.0)
                .build()?;

            let scale = win.scale_factor()?;
            let size = win.inner_size()?.to_logical::<f64>(scale);
            let w = size.width;
            let h = size.height;

            // ブラウザ Webview（全高、UIが上に乗る）
            let app_handle = app.handle().clone();
            win.add_child(
                WebviewBuilder::new(
                    BROWSER_LABEL,
                    WebviewUrl::External("about:blank".parse().unwrap()),
                )
                .on_page_load(move |wv, payload| {
                    let url = payload.url().to_string();
                    if url == "about:blank" {
                        return;
                    }
                    match payload.event() {
                        PageLoadEvent::Started => {
                            if let Some(ui) = app_handle.get_webview(UI_LABEL) {
                                let _ = ui.emit("page-load-start", url);
                            }
                        }
                        PageLoadEvent::Finished => {
                            if let Some(ui) = app_handle.get_webview(UI_LABEL) {
                                let _ = ui.emit("page-load-finish", url);
                            }
                            // リソースタイミングを収集してバックエンドに送る
                            let _ = wv.eval(r#"
                                (function() {
                                    const entries = performance.getEntriesByType('resource').map(e => ({
                                        name: e.name,
                                        duration: Math.round(e.duration),
                                        transfer_size: e.transferSize || 0,
                                        initiator_type: e.initiatorType,
                                    }));
                                    window.__TAURI__.core.invoke('report_resources', { resources: entries });
                                })();
                            "#);
                        }
                    }
                }),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(w, h),
            )?;

            // UI Webview（下端22pxの帯）
            win.add_child(
                WebviewBuilder::new(UI_LABEL, WebviewUrl::App("index.html".into()))
                    .transparent(true),
                LogicalPosition::new(0.0, h - STATUS_H),
                LogicalSize::new(w, STATUS_H),
            )?;

            // リサイズ追従
            let app_handle_resize = app.handle().clone();
            let app_handle_focus = app.handle().clone();
            win.on_window_event(move |event| {
                match event {
                    WindowEvent::Resized(_) => {
                        let app = &app_handle_resize;
                        if let Some(win) = app.get_window("main")
                            && let (Ok(scale), Ok(phys)) = (win.scale_factor(), win.inner_size()) {
                                let s = phys.to_logical::<f64>(scale);
                                if let Some(browser) = app.get_webview(BROWSER_LABEL) {
                                    let _ = browser.set_size(LogicalSize::new(s.width, s.height));
                                }
                                if let Some(ui) = app.get_webview(UI_LABEL) {
                                    let _ = ui
                                        .set_position(LogicalPosition::new(0.0, s.height - STATUS_H));
                                    let _ = ui.set_size(LogicalSize::new(s.width, STATUS_H));
                                }
                            }
                    }
                    WindowEvent::Focused(true) => {
                        let shortcut = Shortcut::new(Some(Modifiers::SHIFT), Code::Semicolon);
                        if !app_handle_focus.global_shortcut().is_registered(shortcut) {
                            let app_handle_colon = app_handle_focus.clone();
                            let _ = app_handle_focus.global_shortcut().on_shortcut(
                                shortcut,
                                move |_, _, _| {
                                    if let Some(ui) = app_handle_colon.get_webview(UI_LABEL) {
                                        let _ = ui.set_focus();
                                        let _ = ui.emit("open-dialog", ());
                                    }
                                },
                            );
                        }
                    }
                    WindowEvent::Focused(false) => {
                        let shortcut = Shortcut::new(Some(Modifiers::SHIFT), Code::Semicolon);
                        if app_handle_focus.global_shortcut().is_registered(shortcut) {
                            let _ = app_handle_focus.global_shortcut().unregister(shortcut);
                        }
                    }
                    _ => {}
                }
            });

            if win.is_focused().unwrap_or(true) {
                let shortcut = Shortcut::new(Some(Modifiers::SHIFT), Code::Semicolon);
                if !app.global_shortcut().is_registered(shortcut) {
                    let app_handle_colon = app.handle().clone();
                    let _ = app.global_shortcut().on_shortcut(shortcut, move |_, _, _| {
                        if let Some(ui) = app_handle_colon.get_webview(UI_LABEL) {
                            let _ = ui.set_focus();
                            let _ = ui.emit("open-dialog", ());
                        }
                    });
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
