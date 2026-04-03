"use strict";

const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.core;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MODE = Object.freeze({ NORMAL: "NORMAL", COMMAND: "COMMAND" });
const SPINNER_FRAMES = Object.freeze(["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"]);
const SPINNER_INTERVAL = 80;
const FIDGET_INTERVAL = 120;
const JJ_THRESHOLD = 400;

// ---------------------------------------------------------------------------
// DOM refs (fail-fast if missing)
// ---------------------------------------------------------------------------

const $ = (id) => {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing element #${id}`);
  return el;
};

const dom = Object.freeze({
  modeIndicator: $("mode-indicator"),
  dot:           $("dot"),
  urlText:       $("url-text"),
  display:       $("display"),
  inputMode:     $("input-mode"),
  urlInput:      $("url-input"),
  buffers:       $("buffers"),
});

// ---------------------------------------------------------------------------
// Spinner — loading animation on the dot indicator
// ---------------------------------------------------------------------------

class Spinner {
  #idx = 0;
  #timer = null;

  start() {
    if (this.#timer) return;
    this.#timer = setInterval(() => {
      dom.dot.textContent = SPINNER_FRAMES[this.#idx++ % SPINNER_FRAMES.length];
    }, SPINNER_INTERVAL);
  }

  stop() {
    clearInterval(this.#timer);
    this.#timer = null;
    dom.dot.textContent = "●";
    dom.dot.className = "";
  }

  get running() {
    return this.#timer !== null;
  }
}

const spinner = new Spinner();

// ---------------------------------------------------------------------------
// FidgetQueue — scrolling resource info in the URL area
// ---------------------------------------------------------------------------

class FidgetQueue {
  #queue = [];
  #timer = null;

  clear() {
    this.#queue = [];
    clearTimeout(this.#timer);
    this.#timer = null;
  }

  enqueue(entries) {
    this.#queue = entries;
    if (!spinner.running) this.#flush();
  }

  tryFlush() {
    if (this.#queue.length > 0) this.#flush();
  }

  #flush() {
    if (this.#timer) return;
    const next = () => {
      if (this.#queue.length === 0) {
        dom.urlText.textContent = state.currentUrl;
        this.#timer = null;
        return;
      }
      dom.urlText.textContent = this.#queue.shift();
      this.#timer = setTimeout(next, FIDGET_INTERVAL);
    };
    next();
  }
}

const fidget = new FidgetQueue();

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

const state = {
  mode: MODE.NORMAL,
  buffers: [],
  activeBuffer: 0,
  currentUrl: "",
  inputOpen: false,
  lastJ: 0,
};

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

function renderMode() {
  dom.modeIndicator.textContent = state.mode;
  dom.modeIndicator.className = state.mode === MODE.COMMAND ? "command" : "normal";
}

function renderBuffers() {
  while (dom.buffers.firstChild) {
    dom.buffers.removeChild(dom.buffers.firstChild);
  }
  for (let i = 0; i < state.buffers.length; i++) {
    const buf = state.buffers[i];
    const el = document.createElement("span");
    el.className = i === state.activeBuffer ? "buf active" : "buf";
    let label = buf.title;
    if (!label) {
      try { label = new URL(buf.url).hostname; } catch { label = buf.url; }
    }
    el.textContent = `${i + 1}:${label}`;
    dom.buffers.appendChild(el);
  }
}

function applyState(snap) {
  state.mode = snap.mode;
  state.buffers = snap.buffers;
  state.activeBuffer = snap.active;
  renderMode();
  renderBuffers();
}

// ---------------------------------------------------------------------------
// Resource formatting
// ---------------------------------------------------------------------------

function formatSize(bytes) {
  if (!bytes) return "";
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / 1048576).toFixed(1)}MB`;
}

function formatEntry(e) {
  try {
    const host = new URL(e.name).hostname;
    const size = formatSize(e.transfer_size);
    const ms = e.duration > 0 ? `${e.duration}ms` : "";
    return [host, e.initiator_type, ms, size].filter(Boolean).join(" ");
  } catch {
    return e.name.slice(0, 40);
  }
}

// ---------------------------------------------------------------------------
// Backend events
// ---------------------------------------------------------------------------

listen("state-change", (e) => applyState(e.payload));

listen("page-load-start", (e) => {
  state.currentUrl = e.payload;
  dom.dot.className = "loading";
  fidget.clear();
  dom.urlText.textContent = e.payload;
  spinner.start();
});

listen("page-load-finish", (e) => {
  state.currentUrl = e.payload;
  dom.urlText.textContent = e.payload;
  setTimeout(() => {
    spinner.stop();
    fidget.tryFlush();
  }, 200);
  if (!state.inputOpen) {
    dom.display.style.display = "flex";
  }
});

listen("resource-log", (e) => {
  const sorted = e.payload
    .filter((r) => r.transfer_size > 0 || r.duration > 0)
    .sort((a, b) => b.transfer_size - a.transfer_size)
    .slice(0, 20)
    .map(formatEntry);
  fidget.enqueue(sorted);
});

// ---------------------------------------------------------------------------
// URL input mode
// ---------------------------------------------------------------------------

function openInput() {
  state.inputOpen = true;
  dom.display.style.display = "none";
  dom.inputMode.classList.add("active");
  dom.urlInput.value = "https://";
  dom.urlInput.focus();
  dom.urlInput.setSelectionRange(dom.urlInput.value.length, dom.urlInput.value.length);
}

function closeInput() {
  state.inputOpen = false;
  dom.inputMode.classList.remove("active");
  dom.display.style.display = "flex";
}

dom.inputMode.addEventListener("submit", (e) => {
  e.preventDefault();
  const url = dom.urlInput.value.trim();
  closeInput();
  if (url && url !== "https://") {
    invoke("navigate_to", { url });
  }
});

dom.urlInput.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    e.preventDefault();
    e.stopPropagation();
    closeInput();
  }
});

// ---------------------------------------------------------------------------
// Keybindings (UI webview — both modes)
// ---------------------------------------------------------------------------

const COMMAND_KEYS = Object.freeze({
  ":"() { openInput(); },
  l()  { invoke("buffer_next"); },
  h()  { invoke("buffer_prev"); },
});

document.addEventListener("keydown", (e) => {
  if (state.inputOpen || e.isComposing) return;

  // Esc → toggle mode (both modes)
  if (e.key === "Escape") {
    e.preventDefault();
    invoke("toggle_mode");
    return;
  }

  // jj → toggle mode (both modes, 400ms window)
  if (e.key === "j") {
    const now = Date.now();
    if (now - state.lastJ < JJ_THRESHOLD) {
      state.lastJ = 0;
      e.preventDefault();
      invoke("toggle_mode");
    } else {
      state.lastJ = now;
    }
    return;
  }

  // COMMAND-only keys
  if (state.mode !== MODE.COMMAND) return;
  const handler = COMMAND_KEYS[e.key];
  if (handler) {
    e.preventDefault();
    handler();
  }
}, true);

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

invoke("get_state").then(applyState).catch(() => {});
