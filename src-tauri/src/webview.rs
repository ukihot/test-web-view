use tauri::webview::PageLoadPayload;
use tauri::{AppHandle, Webview};

use crate::{constants::UI_LABEL, helpers::emit_to_ui, state::ManagedState};

/// `on_page_load` コールバックの本体。
/// ページ読み込みイベントを処理し、IPC プローブや各種レポートを実行する。
pub fn handle_page_load(app_handle: &AppHandle, wv: &Webview, payload: &PageLoadPayload<'_>) {
    use tauri::Manager;
    use tauri::webview::PageLoadEvent;

    let url = payload.url().to_string();
    if url == "about:blank" {
        return;
    }

    match payload.event() {
        PageLoadEvent::Started => {
            emit_to_ui(app_handle, "page-load-start", &url);
        }
        PageLoadEvent::Finished => {
            emit_to_ui(app_handle, "page-load-finish", &url);

            // Probe IPC availability after page load.
            // Error pages (e.g. ERR_NAME_NOT_RESOLVED) lack
            // the __TAURI__ bridge, so the ping will not arrive
            // and we fall back to focusing the UI webview.
            if let Ok(mut guard) = app_handle.state::<ManagedState>().lock_or_err() {
                guard.browser_ipc_ok = false;
            }
            let _ = wv.eval(
                "(function(){try{if(window.__TAURI__&&window.__TAURI__.core){window.__TAURI__.core.invoke('browser_ping').catch(function(){})}}catch(_){}})()",
            );
            let probe_handle = app_handle.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(200));
                let managed = probe_handle.state::<ManagedState>();
                if let Ok(guard) = managed.lock_or_err()
                    && guard.browser_ipc_ok
                {
                    return;
                }
                if let Some(ui) = probe_handle.get_webview(UI_LABEL) {
                    let _ = ui.set_focus();
                }
            });

            eval_post_load_scripts(wv);
        }
    }
}

/// ページ読み込み完了後に実行するスクリプト群。
fn eval_post_load_scripts(wv: &Webview) {
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
    let _ = wv.eval(concat!(
        "(function(){",
        "  if(!window.__TAURI__?.core) return;",
        "  var RE=/token|session|auth|jwt|sid|csrf|api.?key|access|bearer|refresh|sso|oidc|saml/i;",
        "  var t=[];",
        "  try{(document.cookie||'').split(';').forEach(function(c){",
        "    var n=c.split('=')[0].trim(); if(RE.test(n)) t.push('c:'+n);",
        "  });}catch(_){}",
        "  try{for(var i=0;i<localStorage.length;i++){",
        "    var k=localStorage.key(i); if(RE.test(k)) t.push('ls:'+k);",
        "  }}catch(_){}",
        "  try{for(var i=0;i<sessionStorage.length;i++){",
        "    var k=sessionStorage.key(i); if(RE.test(k)) t.push('ss:'+k);",
        "  }}catch(_){}",
        "  window.__TAURI__.core.invoke('report_auth_tokens',",
        "    {tokens:t}).catch(function(){});",
        "})();"
    ));
}
