use std::sync::Mutex;

use crate::domain::{Buffer, Mode, Snapshot};

pub struct AppState {
    pub mode: Mode,
    pub buffers: Vec<Buffer>,
    pub active: usize,
    pub next_id: usize,
}

impl AppState {
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            mode: self.mode,
            buffers: self.buffers.clone(),
            active: self.active,
        }
    }

    pub fn toggle_mode(&mut self) -> Snapshot {
        self.mode = self.mode.toggle();
        self.snapshot()
    }

    pub fn add_buffer(&mut self, url: String) -> Snapshot {
        self.buffers.push(Buffer {
            id: self.next_id,
            url,
            title: String::new(),
        });
        self.next_id += 1;
        self.active = self.buffers.len() - 1;
        self.mode = Mode::Normal;
        self.snapshot()
    }

    pub fn cycle_buffer(&mut self, delta: isize) -> Option<(Snapshot, String)> {
        let len = self.buffers.len();
        if len == 0 {
            return None;
        }
        self.active = ((self.active as isize + delta).rem_euclid(len as isize)) as usize;
        let url = self.buffers[self.active].url.clone();
        Some((self.snapshot(), url))
    }

    pub fn set_active_title(&mut self, title: String) -> Snapshot {
        if let Some(buf) = self.buffers.get_mut(self.active) {
            buf.title = title;
        }
        self.snapshot()
    }
}

pub struct ManagedState(pub Mutex<AppState>);

impl ManagedState {
    pub fn lock_or_err(&self) -> Result<std::sync::MutexGuard<'_, AppState>, String> {
        self.0
            .lock()
            .map_err(|e| format!("state lock poisoned: {e}"))
    }
}
