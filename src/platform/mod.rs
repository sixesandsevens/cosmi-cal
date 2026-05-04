// SPDX-License-Identifier: MPL-2.0

pub mod generic;
pub mod summon;
pub mod x11;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    X11,
    Wayland,
    Unknown,
}

pub fn detect_session() -> SessionKind {
    session_kind_from_str(&std::env::var("XDG_SESSION_TYPE").unwrap_or_default())
}

fn session_kind_from_str(session_type: &str) -> SessionKind {
    match session_type.to_lowercase().as_str() {
        "x11" => SessionKind::X11,
        "wayland" => SessionKind::Wayland,
        _ => SessionKind::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{SessionKind, session_kind_from_str};

    #[test]
    fn detects_unknown_when_session_type_is_empty() {
        assert_eq!(session_kind_from_str(""), SessionKind::Unknown);
    }

    #[test]
    fn detects_x11_case_insensitively() {
        assert_eq!(session_kind_from_str("X11"), SessionKind::X11);
    }
}
