use tauri::{
    LogicalPosition, LogicalSize, Manager, WebviewUrl, WindowEvent,
    webview::{PageLoadEvent, WebviewBuilder},
    window::WindowBuilder,
};

use crate::{
    commands,
    constants::{BROWSER_LABEL, STATUS_H, UI_LABEL},
    domain::{Buffer, Mode},
    helpers::emit_to_ui,
    scripts::BROWSER_INIT_SCRIPT,
    state::{AppState, ManagedState},
};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ManagedState(std::sync::Mutex::new(AppState {
            mode: Mode::default(),
            buffers: vec![Buffer {
                id: 1,
                url: "about:blank".to_owned(),
                title: "about:blank".to_owned(),
            }],
            active: 0,
            next_id: 2,
        })))
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::toggle_mode,
            commands::navigate_to,
            commands::buffer_next,
            commands::buffer_prev,
            commands::close_current_buffer,
            commands::report_title,
            commands::report_resources,
        ])
        .setup(|app| {
            let win = WindowBuilder::new(app, "main")
                .title("vim-browser")
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
                .on_page_load(move |wv, payload| {
                    let url = payload.url().to_string();
                    if url == "about:blank" {
                        return;
                    }
                    match payload.event() {
                        PageLoadEvent::Started => {
                            emit_to_ui(&app_handle, "page-load-start", &url);
                        }
                        PageLoadEvent::Finished => {
                            emit_to_ui(&app_handle, "page-load-finish", &url);
                            let _ = wv.eval(concat!(
                                "(function(){",
                                "  if(!window.__TAURI__?.core) return;",
                                "  window.__TAURI__.core.invoke('report_title',",
                                "    {title:document.title||''}).catch(function(){});",
                                "})();"
                            ));
                            let _ = wv.eval(concat!(
                                "(function(){",
                                "  if(!window.__TAURI__?.core) return;",
                                "  var entries=performance.getEntriesByType('resource').map(function(e){",
                                "    return{name:e.name,duration:Math.round(e.duration),",
                                "      transfer_size:e.transferSize||0,initiator_type:e.initiatorType};",
                                "  });",
                                "  window.__TAURI__.core.invoke('report_resources',",
                                "    {resources:entries}).catch(function(){});",
                                "})();"
                            ));
                        }
                    }
                }),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(w, h),
            )?;

            // UI webview status bar at bottom.
            win.add_child(
                WebviewBuilder::new(UI_LABEL, WebviewUrl::App("index.html".into())).transparent(true),
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
            if let Some(ui) = app.get_webview(UI_LABEL) {
                let _ = ui.set_focus();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
