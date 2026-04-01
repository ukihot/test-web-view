use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlIFrameElement, HtmlInputElement, KeyboardEvent, window};

#[component]
pub fn App() -> impl IntoView {
    let (url, set_url) = signal("https://example.com".to_string());
    let (url_input, set_url_input) = signal(String::new());
    let (mode, set_mode) = signal("NORMAL".to_string());
    let (status, set_status) =
        signal("VIM 操作: h/j/k/l, gg, G, d, u, : (コマンドモード)".to_string());
    let (last_key, set_last_key) = signal(None::<String>);
    let (is_loading, set_is_loading) = signal(true);

    let get_iframe = move || {
        window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("web-view"))
            .and_then(|e| e.dyn_into::<HtmlIFrameElement>().ok())
    };

    let apply_scroll = move |dx: f64, dy: f64| {
        if let Some(iframe) = get_iframe()
            && let Some(content_window) = iframe.content_window()
        {
            content_window.scroll_by_with_x_and_y(dx, dy);
        }
    };

    let scroll_top = move || {
        if let Some(iframe) = get_iframe()
            && let Some(content_window) = iframe.content_window()
        {
            content_window.scroll_to_with_x_and_y(0.0, 0.0);
        }
    };

    let scroll_bottom = move || {
        if let Some(iframe) = get_iframe()
            && let Some(content_window) = iframe.content_window()
            && let Some(doc) = iframe.content_document()
            && let Some(body) = doc.body()
        {
            let y = body.scroll_height() as f64;
            content_window.scroll_to_with_x_and_y(0.0, y);
        }
    };

    // 常にキーボードを拾うために window でイベントリスナを登録
    Effect::new(move |_| {
        let window = window().expect("no window");
        let window_for_closure = window.clone();
        let mode = mode;
        let last_key = last_key;
        let set_mode = set_mode;
        let set_status = set_status;
        let set_last_key = set_last_key;

        let cl = Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            let window = window_for_closure.clone();
            let key = event.key();
            if key.is_empty() {
                return;
            }

            // フォーカスがアドレス入力中の場合はノーマル操作を無視
            if let Some(active) = window.document().and_then(|d| d.active_element())
                && active.tag_name().to_lowercase() == "input"
            {
                if key == "Escape" {
                    set_mode.set("NORMAL".to_string());
                    set_status.set("NORMAL モード".to_string());
                }
                return;
            }

            if mode.get() != "NORMAL" {
                return;
            }

            match key.as_str() {
                "h" => {
                    apply_scroll(-80.0, 0.0);
                    set_status.set("← 左スクロール".to_string());
                }
                "l" => {
                    apply_scroll(80.0, 0.0);
                    set_status.set("→ 右スクロール".to_string());
                }
                "j" => {
                    apply_scroll(0.0, 80.0);
                    set_status.set("↓ 下スクロール".to_string());
                }
                "k" => {
                    apply_scroll(0.0, -80.0);
                    set_status.set("↑ 上スクロール".to_string());
                }
                "d" => {
                    apply_scroll(0.0, 400.0);
                    set_status.set("Page Down".to_string());
                }
                "u" => {
                    apply_scroll(0.0, -400.0);
                    set_status.set("Page Up".to_string());
                }
                "g" => {
                    if last_key.get().as_deref() == Some("g") {
                        scroll_top();
                        set_status.set("Top".to_string());
                        set_last_key.set(None);
                    } else {
                        set_last_key.set(Some("g".to_string()));
                    }
                }
                "G" => {
                    scroll_bottom();
                    set_status.set("Bottom".to_string());
                }
                ":" => {
                    set_mode.set("COMMAND".to_string());
                    set_status.set("コマンドモード: アドレス欄にURLを入力してEnter".to_string());
                    if let Some(elem) = window
                        .document()
                        .and_then(|d| d.get_element_by_id("address-input"))
                    {
                        let _ = elem.dyn_ref::<HtmlInputElement>().map(|i| i.focus());
                    }
                }
                _ => {
                    set_last_key.set(None);
                }
            }

            if key != "g" {
                set_last_key.set(None);
            }
            event.prevent_default();
        });

        window
            .add_event_listener_with_callback("keydown", cl.as_ref().unchecked_ref())
            .unwrap();
        move || {
            window
                .remove_event_listener_with_callback("keydown", cl.as_ref().unchecked_ref())
                .unwrap();
        }
    });

    let update_url_input = move |event: web_sys::Event| {
        if let Some(input) = event
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            set_url_input.set(input.value());
        }
    };

    let submit_url = move |ev: SubmitEvent| {
        ev.prevent_default();
        let raw = url_input.get_untracked();
        if raw.trim().is_empty() {
            return;
        }
        let normalized = if raw.starts_with("http://") || raw.starts_with("https://") {
            raw.clone()
        } else {
            format!("https://{}", raw)
        };
        set_url.set(normalized);
        set_url_input.set(String::new());
        set_mode.set("NORMAL".to_string());
        set_status.set("NORMAL モード".to_string());
        set_is_loading.set(true);
    };

    view! {
        <main class="container browser-container">
            <form class="address-bar" on:submit=submit_url>
                <input
                    id="address-input"
                    type="text"
                    placeholder="https://example.com"
                    on:input=update_url_input
                    prop:value=url_input
                />
                <button type="submit">"Go"</button>
            </form>
            <div class="webview-shell" tabindex="0">
                <iframe
                    id="web-view"
                    src={move || url.get()}
                    class="webview-iframe"
                    on:load={move |_| set_is_loading.set(false)}
                />
                { move || view! {
                    <div
                        class="loading-overlay"
                        style={move || if is_loading.get() { "display:flex" } else { "display:none" }}
                    >
                        "Loading..."
                    </div>
                }}
            </div>
            <footer class="status-line">
                <span>{ move || format!("{} | Mode: {}  URL: {}", status.get(), mode.get(), url.get()) }</span>
            </footer>
        </main>
    }
}
