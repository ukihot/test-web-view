use std::sync::Mutex;

use crate::domain::{Buffer, Mode, Snapshot};

pub struct AppState {
    pub mode: Mode,
    pub buffers: Vec<Buffer>,
    pub active: usize,
    pub next_id: usize,
    pub browser_ipc_ok: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn new_state() -> AppState {
        AppState {
            mode: Mode::default(),
            buffers: vec![Buffer {
                id: 1,
                url: "about:blank".to_owned(),
                title: "about:blank".to_owned(),
            }],
            active: 0,
            next_id: 2,
            browser_ipc_ok: false,
        }
    }

    // -- snapshot ---------------------------------------------------------

    #[test]
    fn snapshot_reflects_current_state() {
        let st = new_state();
        let snap = st.snapshot();
        assert_eq!(snap.mode, Mode::Normal);
        assert_eq!(snap.active, 0);
        assert_eq!(snap.buffers.len(), 1);
    }

    // -- toggle_mode ------------------------------------------------------

    #[test]
    fn toggle_mode_normal_to_command() {
        let mut st = new_state();
        let snap = st.toggle_mode();
        assert_eq!(snap.mode, Mode::Command);
    }

    #[test]
    fn toggle_mode_command_to_normal() {
        let mut st = new_state();
        st.mode = Mode::Command;
        let snap = st.toggle_mode();
        assert_eq!(snap.mode, Mode::Normal);
    }

    // -- enter_command ----------------------------------------------------

    #[test]
    fn enter_command_from_normal_succeeds() {
        let mut st = new_state();
        let snap = st.enter_command();
        assert!(snap.is_some());
        assert_eq!(snap.unwrap().mode, Mode::Command);
    }

    #[test]
    fn enter_command_from_command_is_noop() {
        let mut st = new_state();
        st.mode = Mode::Command;
        assert!(st.enter_command().is_none());
    }

    // -- enter_normal -----------------------------------------------------

    #[test]
    fn enter_normal_from_command_succeeds() {
        let mut st = new_state();
        st.mode = Mode::Command;
        let snap = st.enter_normal();
        assert!(snap.is_some());
        assert_eq!(snap.unwrap().mode, Mode::Normal);
    }

    #[test]
    fn enter_normal_from_normal_is_noop() {
        let mut st = new_state();
        assert!(st.enter_normal().is_none());
    }

    // -- add_buffer -------------------------------------------------------

    #[test]
    fn add_buffer_appends_and_activates() {
        let mut st = new_state();
        let snap = st.add_buffer("https://example.com".to_owned());
        assert_eq!(snap.buffers.len(), 2);
        assert_eq!(snap.active, 1);
        assert_eq!(snap.buffers[1].url, "https://example.com");
    }

    #[test]
    fn add_buffer_increments_id() {
        let mut st = new_state();
        st.add_buffer("https://a.com".to_owned());
        assert_eq!(st.next_id, 3);
        st.add_buffer("https://b.com".to_owned());
        assert_eq!(st.next_id, 4);
    }

    #[test]
    fn add_buffer_resets_mode_to_normal() {
        let mut st = new_state();
        st.mode = Mode::Command;
        let snap = st.add_buffer("https://x.com".to_owned());
        assert_eq!(snap.mode, Mode::Normal);
    }

    // -- navigate_active ---------------------------------------------------

    #[test]
    fn navigate_active_updates_url() {
        let mut st = new_state();
        let snap = st.navigate_active("https://rust-lang.org".to_owned());
        assert_eq!(snap.buffers[0].url, "https://rust-lang.org");
        assert!(snap.buffers[0].title.is_empty());
    }

    #[test]
    fn navigate_active_resets_mode_to_normal() {
        let mut st = new_state();
        st.mode = Mode::Command;
        let snap = st.navigate_active("https://example.com".to_owned());
        assert_eq!(snap.mode, Mode::Normal);
    }

    #[test]
    fn navigate_active_creates_buffer_if_empty() {
        let mut st = AppState {
            mode: Mode::Normal,
            buffers: vec![],
            active: 0,
            next_id: 1,
            browser_ipc_ok: false,
        };
        let snap = st.navigate_active("https://new.com".to_owned());
        assert_eq!(snap.buffers.len(), 1);
        assert_eq!(snap.buffers[0].url, "https://new.com");
    }

    // -- cycle_buffer -----------------------------------------------------

    #[test]
    fn cycle_buffer_forward() {
        let mut st = new_state();
        st.add_buffer("https://a.com".to_owned());
        st.add_buffer("https://b.com".to_owned());
        st.active = 0;
        let (snap, url) = st.cycle_buffer(1).unwrap();
        assert_eq!(snap.active, 1);
        assert_eq!(url, "https://a.com");
    }

    #[test]
    fn cycle_buffer_wraps_around() {
        let mut st = new_state();
        st.add_buffer("https://a.com".to_owned());
        // active is now 1 (last added)
        let (snap, url) = st.cycle_buffer(1).unwrap();
        assert_eq!(snap.active, 0);
        assert_eq!(url, "about:blank");
    }

    #[test]
    fn cycle_buffer_backward() {
        let mut st = new_state();
        st.add_buffer("https://a.com".to_owned());
        st.active = 0;
        let (snap, _) = st.cycle_buffer(-1).unwrap();
        assert_eq!(snap.active, 1);
    }

    #[test]
    fn cycle_buffer_empty_returns_none() {
        let mut st = AppState {
            mode: Mode::Normal,
            buffers: vec![],
            active: 0,
            next_id: 1,
            browser_ipc_ok: false,
        };
        assert!(st.cycle_buffer(1).is_none());
    }

    // -- set_active_title -------------------------------------------------

    #[test]
    fn set_active_title_updates_title() {
        let mut st = new_state();
        let snap = st.set_active_title("My Page".to_owned());
        assert_eq!(snap.buffers[0].title, "My Page");
    }

    // -- close_active_buffer ----------------------------------------------

    #[test]
    fn close_last_buffer_resets_to_blank() {
        let mut st = new_state();
        let (snap, url) = st.close_active_buffer();
        assert_eq!(snap.buffers.len(), 1);
        assert_eq!(snap.buffers[0].url, "about:blank");
        assert_eq!(url, "about:blank");
    }

    #[test]
    fn close_buffer_removes_and_navigates() {
        let mut st = new_state();
        st.add_buffer("https://a.com".to_owned());
        st.add_buffer("https://b.com".to_owned());
        // active = 2 (last added, index 2)
        let (snap, url) = st.close_active_buffer();
        assert_eq!(snap.buffers.len(), 2);
        // should navigate to the previous buffer
        assert!(!url.is_empty());
    }

    #[test]
    fn close_buffer_adjusts_active_index() {
        let mut st = new_state();
        st.add_buffer("https://a.com".to_owned());
        // active = 1
        let (snap, _) = st.close_active_buffer();
        assert_eq!(snap.active, 0);
    }

    #[test]
    fn close_empty_buffers_creates_blank() {
        let mut st = AppState {
            mode: Mode::Normal,
            buffers: vec![],
            active: 0,
            next_id: 1,
            browser_ipc_ok: false,
        };
        let (snap, url) = st.close_active_buffer();
        assert_eq!(snap.buffers.len(), 1);
        assert_eq!(url, "about:blank");
    }

    // -- ManagedState -----------------------------------------------------

    #[test]
    fn managed_state_lock_succeeds() {
        let ms = ManagedState(Mutex::new(new_state()));
        assert!(ms.lock_or_err().is_ok());
    }
}
