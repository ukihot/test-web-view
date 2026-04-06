use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Mode {
    #[default]
    Normal,
    Command,
}

impl Mode {
    pub const fn toggle(self) -> Self {
        match self {
            Self::Normal => Self::Command,
            Self::Command => Self::Normal,
        }
    }

    pub const fn is_normal(self) -> bool {
        matches!(self, Self::Normal)
    }

    pub const fn is_command(self) -> bool {
        matches!(self, Self::Command)
    }
}

#[derive(Clone, Serialize)]
pub struct Buffer {
    pub id: usize,
    pub url: String,
    pub title: String,
}

#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub mode: Mode,
    pub buffers: Vec<Buffer>,
    pub active: usize,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ResourceEntry {
    pub name: String,
    pub duration: f64,
    pub transfer_size: f64,
    pub initiator_type: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ActivityEntry {
    pub kind: String,
    pub detail: String,
    pub direction: String,
    pub timestamp: f64,
}
