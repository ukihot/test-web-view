use tauri::{
    LogicalPosition, LogicalSize, Manager, WebviewUrl, WindowEvent, webview::WebviewBuilder,
    window::WindowBuilder,
};

use crate::{
    commands,
    constants::{BROWSER_LABEL, ENABLE_UPDATER, STATUS_H, UI_LABEL},
    domain::{Buffer, Mode},
    scripts::{ACTIVITY_INIT_SCRIPT, BROWSER_INIT_SCRIPT},
    state::{AppState, ManagedState},
    updater, webview,
};

pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init());

    if ENABLE_UPDATER {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .manage(ManagedState(std::sync::Mutex::new(AppState {
            mode: Mode::default(),
            buffers: vec![Buffer {
                id: 1,
                url: "about:blank".to_owned(),
                title: "about:blank".to_owned(),
            }],
            active: 0,
            next_id: 2,
            browser_ipc_ok: false,
            update_tx: None,
        })))
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::browser_ping,
            commands::toggle_mode,
            commands::enter_command,
            commands::enter_normal,
            commands::navigate_to,
            commands::buffer_next,
            commands::buffer_prev,
            commands::close_current_buffer,
            commands::report_title,
            commands::report_resources,
            commands::report_activity,
            commands::report_auth_tokens,
            commands::respond_update,
        ])
        .setup(|app| {
            let version = app.config().version.clone().unwrap_or_default();
            let win = WindowBuilder::new(app, "main")
                .title(format!("vim-browser v{version}"))
                .inner_size(1200.0, 800.0)
                .build()?;

            let scale = win.scale_factor()?;
            let size = win.inner_size()?.to_logical::<f64>(scale);
            let (w, h) = (size.width, size.height);

            // Browser webview rendered beneath UI.
            let app_handle = app.handle().clone();
            win.add_child(
                WebviewBuilder::new(
                    BROWSER_LABEL,
                    WebviewUrl::External("about:blank".parse().unwrap()),
                )
                .initialization_script(BROWSER_INIT_SCRIPT)
                .initialization_script(ACTIVITY_INIT_SCRIPT)
                .on_page_load(move |wv, payload| {
                    webview::handle_page_load(&app_handle, &wv, &payload);
                }),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(w, h),
            )?;

            // UI webview status bar at bottom.
            win.add_child(
                WebviewBuilder::new(UI_LABEL, WebviewUrl::App("index.html".into()))
                    .transparent(true),
                LogicalPosition::new(0.0, h - STATUS_H),
                LogicalSize::new(w, STATUS_H),
            )?;

            // Resize tracking.
            let resize_handle = app.handle().clone();
            win.on_window_event(move |event| {
                if let WindowEvent::Resized(_) = event
                    && let Some(win) = resize_handle.get_window("main")
                    && let (Ok(scale), Ok(phys)) = (win.scale_factor(), win.inner_size())
                {
                    let s = phys.to_logical::<f64>(scale);
                    if let Some(browser) = resize_handle.get_webview(BROWSER_LABEL) {
                        let _ = browser.set_size(LogicalSize::new(s.width, s.height));
                    }
                    if let Some(ui) = resize_handle.get_webview(UI_LABEL) {
                        let _ = ui.set_position(LogicalPosition::new(0.0, s.height - STATUS_H));
                        let _ = ui.set_size(LogicalSize::new(s.width, STATUS_H));
                    }
                }
            });

            // 起動直後はUIにフォーカスを当ててキー入力を即受付可能にする
            // webview初期化完了を待つため少し遅延させる
            let focus_handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(300));
                if let Some(ui) = focus_handle.get_webview(UI_LABEL) {
                    let _ = ui.set_focus();
                }
            });

            // バックグラウンドでアップデートチェック
            if ENABLE_UPDATER {
                updater::spawn_update_check(app.handle().clone());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
