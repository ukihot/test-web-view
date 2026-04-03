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

pub const ACTIVITY_INIT_SCRIPT: &str = r#"
(function() {
    "use strict";
    var _t = window.__TAURI__;
    if (!_t || !_t.core) return;
    var invoke = _t.core.invoke;
    var Q = [];
    var tid = 0;
    var IPC_RE = /ipc\.localhost|tauri\.localhost|__TAURI_IPC__/;

    function p(k, d, dir) {
        Q.push({ kind: k, detail: d || "", direction: dir || "", timestamp: performance.now() });
        if (!tid) tid = setTimeout(flush, 200);
    }

    function flush() {
        tid = 0;
        if (!Q.length) return;
        var b = Q.splice(0);
        invoke("report_activity", { entries: b }).catch(function(){});
    }

    // --- WebSocket ---
    var WS = window.WebSocket;
    if (WS) {
        window.WebSocket = function(u, pr) {
            p("ws.new", u, "\u2194");
            var s = pr !== undefined ? new WS(u, pr) : new WS(u);
            s.addEventListener("open", function() { p("ws.open", u, "\u2194"); });
            s.addEventListener("close", function(ev) { p("ws.close", u + " " + ev.code, "\u00d7"); });
            s.addEventListener("message", function(ev) {
                var sz = typeof ev.data === "string" ? ev.data.length + "c" : "bin";
                p("ws.msg", u + " " + sz, "\u2190");
            });
            var _send = s.send.bind(s);
            s.send = function(d) {
                var sz = typeof d === "string" ? d.length + "c" : "bin";
                p("ws.send", u + " " + sz, "\u2192");
                return _send(d);
            };
            return s;
        };
        window.WebSocket.prototype = WS.prototype;
        window.WebSocket.CONNECTING = 0;
        window.WebSocket.OPEN = 1;
        window.WebSocket.CLOSING = 2;
        window.WebSocket.CLOSED = 3;
    }

    // --- fetch ---
    var _fetch = window.fetch;
    window.fetch = function(input, init) {
        var u = typeof input === "string" ? input : (input && input.url ? input.url : "?");
        if (IPC_RE.test(u)) return _fetch.apply(this, arguments);
        var m = init && init.method ? init.method.toUpperCase() : "GET";
        p("fetch", m + " " + u, "\u2192");
        return _fetch.apply(this, arguments).then(function(r) {
            p("fetch." + r.status, u, "\u2190");
            return r;
        }).catch(function(e) {
            p("fetch.err", u + " " + e.message, "\u00d7");
            throw e;
        });
    };

    // --- XMLHttpRequest ---
    var _xOpen = XMLHttpRequest.prototype.open;
    var _xSend = XMLHttpRequest.prototype.send;
    XMLHttpRequest.prototype.open = function(m, u) {
        this._am = m; this._au = u;
        return _xOpen.apply(this, arguments);
    };
    XMLHttpRequest.prototype.send = function() {
        var self = this;
        if (IPC_RE.test(self._au || "")) return _xSend.apply(this, arguments);
        p("xhr", (self._am || "?") + " " + (self._au || ""), "\u2192");
        this.addEventListener("loadend", function() {
            p("xhr." + self.status, self._au || "", "\u2190");
        });
        return _xSend.apply(this, arguments);
    };

    // --- sendBeacon ---
    if (navigator.sendBeacon) {
        var _beacon = navigator.sendBeacon.bind(navigator);
        navigator.sendBeacon = function(u) {
            p("beacon", u, "\u2192");
            return _beacon.apply(navigator, arguments);
        };
    }

    // --- EventSource (SSE) ---
    var ES = window.EventSource;
    if (ES) {
        window.EventSource = function(u, o) {
            p("sse.new", u, "\u2190");
            var s = new ES(u, o);
            s.addEventListener("open", function() { p("sse.open", u, "\u2194"); });
            s.addEventListener("error", function() { p("sse.err", u, "\u00d7"); });
            s.addEventListener("message", function() { p("sse.msg", u, "\u2190"); });
            return s;
        };
        window.EventSource.prototype = ES.prototype;
    }

    // --- Service Worker ---
    if (navigator.serviceWorker) {
        navigator.serviceWorker.getRegistrations().then(function(rs) {
            rs.forEach(function(r) { p("sw.reg", r.scope, "\u2194"); });
        }).catch(function(){});
        navigator.serviceWorker.addEventListener("controllerchange", function() {
            p("sw.ctrl", (navigator.serviceWorker.controller||{}).scriptURL||"", "\u2194");
        });
    }

    // --- PerformanceObserver (real-time resource timing) ---
    try {
        new PerformanceObserver(function(list) {
            list.getEntries().forEach(function(e) {
                if (IPC_RE.test(e.name)) return;
                var sz = e.transferSize > 0
                    ? (e.transferSize > 1024 ? (e.transferSize/1024|0)+"K" : e.transferSize+"B")
                    : "";
                var ms = e.duration > 0 ? (e.duration|0) + "ms" : "";
                var info = [ms, sz].filter(Boolean).join(" ");
                p("net." + (e.initiatorType||"other"), e.name + (info ? " " + info : ""), "\u2190");
            });
        }).observe({ type: "resource", buffered: false });
    } catch(_){}

    // --- Storage ---
    try {
        var _setItem = Storage.prototype.setItem;
        Storage.prototype.setItem = function(k) {
            p("store.set", k, "\u2192");
            return _setItem.apply(this, arguments);
        };
    } catch(_){}

    // --- Cookie (CookieStore API) ---
    if (window.cookieStore && window.cookieStore.addEventListener) {
        window.cookieStore.addEventListener("change", function(ev) {
            (ev.changed||[]).forEach(function(c) { p("cookie.set", c.name, "\u2192"); });
            (ev.deleted||[]).forEach(function(c) { p("cookie.del", c.name, "\u00d7"); });
        });
    }
})();
"#;
