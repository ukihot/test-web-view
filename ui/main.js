const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.core;

const dot = document.getElementById('dot');
const urlText = document.getElementById('url-text');
const display = document.getElementById('display');
const inputMode = document.getElementById('input-mode');
const urlInput = document.getElementById('url-input');

// fidget spinner
const SPINNER = ['⠋','⠙','⠹','⠸','⠼','⠴','⠦','⠧','⠇','⠏'];
let spinnerIdx = 0;
let spinnerTimer = null;
let fidgetTimer = null;
let fidgetQueue = [];
let currentUrl = '';

function startSpinner() {
  if (spinnerTimer) return;
  spinnerTimer = setInterval(() => {
    dot.textContent = SPINNER[spinnerIdx++ % SPINNER.length];
  }, 80);
}

function stopSpinner() {
  clearInterval(spinnerTimer);
  spinnerTimer = null;
  dot.textContent = '●';
  dot.className = '';
}

function formatSize(bytes) {
  if (!bytes) return '';
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)}MB`;
}

function formatEntry(e) {
  try {
    const host = new URL(e.name).hostname;
    const size = formatSize(e.transfer_size);
    const ms = e.duration > 0 ? `${e.duration}ms` : '';
    return [host, e.initiator_type, ms, size].filter(Boolean).join(' ');
  } catch {
    return e.name.slice(0, 40);
  }
}

function runFidgetQueue() {
  if (fidgetTimer) return;
  function next() {
    if (fidgetQueue.length === 0) {
      urlText.textContent = currentUrl;
      fidgetTimer = null;
      return;
    }
    urlText.textContent = fidgetQueue.shift();
    fidgetTimer = setTimeout(next, 120);
  }
  next();
}

listen('page-load-start', (e) => {
  currentUrl = e.payload;
  dot.className = 'loading';
  clearTimeout(fidgetTimer);
  fidgetTimer = null;
  fidgetQueue = [];
  urlText.textContent = e.payload;
  startSpinner();
});

listen('page-load-finish', (e) => {
  currentUrl = e.payload;
  urlText.textContent = e.payload;
  setTimeout(() => {
    stopSpinner();
    if (fidgetQueue.length > 0) runFidgetQueue();
  }, 200);
  if (!inputMode.classList.contains('active')) {
    display.style.display = 'flex';
  }
});

listen('resource-log', (e) => {
  const sorted = e.payload
    .filter(r => r.transfer_size > 0 || r.duration > 0)
    .sort((a, b) => b.transfer_size - a.transfer_size)
    .slice(0, 20);
  fidgetQueue = sorted.map(formatEntry);
  if (!spinnerTimer) runFidgetQueue();
});

listen('open-dialog', () => {
  if (!inputMode.classList.contains('active')) openInput();
});

function openInput() {
  display.style.display = 'none';
  inputMode.classList.add('active');
  urlInput.value = 'https://';
  urlInput.focus();
  urlInput.setSelectionRange(urlInput.value.length, urlInput.value.length);
}

function closeInput() {
  inputMode.classList.remove('active');
  display.style.display = 'flex';
}

inputMode.addEventListener('submit', (e) => {
  e.preventDefault();
  const url = urlInput.value.trim();
  closeInput();
  if (url && url !== 'https://') {
    invoke('navigate_to', { url });
  }
});

urlInput.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') closeInput();
});
