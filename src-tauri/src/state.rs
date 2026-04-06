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

    pub fn enter_command(&mut self) -> Option<Snapshot> {
        if self.mode.is_normal() {
            self.mode = Mode::Command;
            Some(self.snapshot())
        } else {
            None
        }
    }

    pub fn enter_normal(&mut self) -> Option<Snapshot> {
        if self.mode.is_command() {
            self.mode = Mode::Normal;
            Some(self.snapshot())
        } else {
            None
        }
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

    pub fn navigate_active(&mut self, url: String) -> Snapshot {
        if self.buffers.is_empty() {
            return self.add_buffer(url);
        }
        if let Some(buf) = self.buffers.get_mut(self.active) {
            buf.url = url;
            buf.title.clear();
        }
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

    pub fn close_active_buffer(&mut self) -> (Snapshot, String) {
        if self.buffers.is_empty() {
            let snap = self.add_buffer("about:blank".to_owned());
            return (snap, "about:blank".to_owned());
        }

        if self.buffers.len() == 1 {
            if let Some(buf) = self.buffers.get_mut(0) {
                buf.url = "about:blank".to_owned();
                buf.title = "about:blank".to_owned();
            }
            self.active = 0;
            let snap = self.snapshot();
            return (snap, "about:blank".to_owned());
        }

        self.buffers.remove(self.active);
        if self.active >= self.buffers.len() {
            self.active = self.buffers.len() - 1;
        }
        let nav_url = self.buffers[self.active].url.clone();
        (self.snapshot(), nav_url)
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
