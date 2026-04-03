pub const BROWSER_INIT_SCRIPT: &str = r#"
(function() {
    "use strict";
    const EDITABLE_TAGS = new Set(["input", "textarea", "select"]);
    const JJ_THRESHOLD = 400;
    let lastJ = 0;

    function tryInvoke(cmd) {
        try {
            if (window.__TAURI__?.core) {
                window.__TAURI__.core.invoke(cmd).catch(function() {});
            }
        } catch (_) { /* IPC unavailable */ }
    }

    document.addEventListener("keydown", function(e) {
        if (e.isComposing) return;
        const tag = (e.target.tagName || "").toLowerCase();
        if (e.target.isContentEditable || EDITABLE_TAGS.has(tag)) return;

        if (e.key === "Escape") {
            e.preventDefault();
            tryInvoke("toggle_mode");
            return;
        }
        if (e.ctrlKey && (e.key === "w" || e.key === "W")) {
            e.preventDefault();
            tryInvoke("close_current_buffer");
            return;
        }
        if (e.key === "j") {
            const now = Date.now();
            if (now - lastJ < JJ_THRESHOLD) {
                lastJ = 0;
                e.preventDefault();
                tryInvoke("toggle_mode");
            } else {
                lastJ = now;
            }
        }
    }, true);
})();
"#;
