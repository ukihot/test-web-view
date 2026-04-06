use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_default_is_normal() {
        assert_eq!(Mode::default(), Mode::Normal);
    }

    #[test]
    fn mode_toggle_normal_to_command() {
        assert_eq!(Mode::Normal.toggle(), Mode::Command);
    }

    #[test]
    fn mode_toggle_command_to_normal() {
        assert_eq!(Mode::Command.toggle(), Mode::Normal);
    }

    #[test]
    fn mode_toggle_roundtrip() {
        assert_eq!(Mode::Normal.toggle().toggle(), Mode::Normal);
    }

    #[test]
    fn mode_is_normal() {
        assert!(Mode::Normal.is_normal());
        assert!(!Mode::Command.is_normal());
    }

    #[test]
    fn mode_is_command() {
        assert!(Mode::Command.is_command());
        assert!(!Mode::Normal.is_command());
    }

    #[test]
    fn mode_serializes_uppercase() {
        assert_eq!(serde_json::to_string(&Mode::Normal).unwrap(), r#""NORMAL""#);
        assert_eq!(
            serde_json::to_string(&Mode::Command).unwrap(),
            r#""COMMAND""#
        );
    }

    #[test]
    fn mode_deserializes_uppercase() {
        let n: Mode = serde_json::from_str(r#""NORMAL""#).unwrap();
        assert_eq!(n, Mode::Normal);
        let c: Mode = serde_json::from_str(r#""COMMAND""#).unwrap();
        assert_eq!(c, Mode::Command);
    }
}
