use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::actions::{NavDir, PageDir};

#[derive(Debug, PartialEq, Eq)]
pub enum UiEvent {
    Quit,

    SwitchTabBrowse,
    SwitchTabSql,
    ToggleFocus,
    CycleTheme,

    Nav(NavDir),
    Page(PageDir),
    OpenSelection,
    Refresh,

    // SQL editor input
    SqlInput(char),
    SqlBackspace,
    SqlNewline,
    SqlMoveCursorLeft,
    SqlMoveCursorRight,
    ExecuteSql,
    ToggleCompletion,
    CompletionNext,
    CompletionPrev,
    AcceptCompletion,
    ConfirmPending,
    CancelPending,
}

pub fn poll_next_event(tick: Duration) -> Result<Option<UiEvent>> {
    if !event::poll(tick)? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(k) => Ok(map_key_event(k)),
        _ => Ok(None),
    }
}

pub fn map_key_event(k: KeyEvent) -> Option<UiEvent> {
    if k.kind != event::KeyEventKind::Press {
        return None;
    }

    let ev = match (k.code, k.modifiers) {
        (KeyCode::Char('q'), _) => UiEvent::Quit,
        (KeyCode::Esc, _) => UiEvent::CancelPending,

        (KeyCode::F(2), _) => UiEvent::SwitchTabBrowse,
        (KeyCode::F(3), _) => UiEvent::SwitchTabSql,
        (KeyCode::Char('t'), KeyModifiers::CONTROL) => UiEvent::CycleTheme,

        (KeyCode::Tab, _) => UiEvent::ToggleFocus,

        (KeyCode::PageUp, _) => UiEvent::Page(PageDir::Prev),
        (KeyCode::PageDown, _) => UiEvent::Page(PageDir::Next),

        (KeyCode::F(5), _) => UiEvent::ExecuteSql,
        (KeyCode::Enter, KeyModifiers::CONTROL) => UiEvent::ExecuteSql,
        (KeyCode::Enter, KeyModifiers::NONE) => UiEvent::OpenSelection,

        (KeyCode::Char('r'), KeyModifiers::CONTROL) => UiEvent::Refresh,

        // SQL editing primitives
        (KeyCode::Backspace, _) => UiEvent::SqlBackspace,
        (KeyCode::Left, _) => UiEvent::SqlMoveCursorLeft,
        (KeyCode::Right, _) => UiEvent::SqlMoveCursorRight,
        (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => UiEvent::SqlInput(c),
        (KeyCode::Enter, _) => UiEvent::SqlNewline,
        (KeyCode::Char(' '), KeyModifiers::CONTROL) => UiEvent::ToggleCompletion,
        (KeyCode::Char('y'), KeyModifiers::CONTROL) => UiEvent::AcceptCompletion,
        (KeyCode::Char('y'), KeyModifiers::NONE) => UiEvent::ConfirmPending,
        (KeyCode::Up, KeyModifiers::CONTROL) => UiEvent::CompletionPrev,
        (KeyCode::Down, KeyModifiers::CONTROL) => UiEvent::CompletionNext,

        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => UiEvent::Nav(NavDir::Up),
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => UiEvent::Nav(NavDir::Down),

        _ => return None,
    };

    Some(ev)
}

#[cfg(test)]
mod tests {
    use super::{map_key_event, UiEvent};
    use crate::app::actions::{NavDir, PageDir};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn maps_execution_keys() {
        assert_eq!(
            map_key_event(key(KeyCode::F(5), KeyModifiers::NONE)),
            Some(UiEvent::ExecuteSql)
        );
        assert_eq!(
            map_key_event(key(KeyCode::F(5), KeyModifiers::CONTROL)),
            Some(UiEvent::ExecuteSql)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Enter, KeyModifiers::CONTROL)),
            Some(UiEvent::ExecuteSql)
        );
    }

    #[test]
    fn maps_plain_enter_to_open_selection() {
        assert_eq!(
            map_key_event(key(KeyCode::Enter, KeyModifiers::NONE)),
            Some(UiEvent::OpenSelection)
        );
    }

    #[test]
    fn maps_navigation_and_paging() {
        assert_eq!(
            map_key_event(key(KeyCode::Up, KeyModifiers::NONE)),
            Some(UiEvent::Nav(NavDir::Up))
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('j'), KeyModifiers::NONE)),
            Some(UiEvent::SqlInput('j'))
        );
        assert_eq!(
            map_key_event(key(KeyCode::PageUp, KeyModifiers::NONE)),
            Some(UiEvent::Page(PageDir::Prev))
        );
        assert_eq!(
            map_key_event(key(KeyCode::PageDown, KeyModifiers::NONE)),
            Some(UiEvent::Page(PageDir::Next))
        );
    }

    #[test]
    fn maps_confirmation_and_completion_keys() {
        assert_eq!(
            map_key_event(key(KeyCode::Char('y'), KeyModifiers::NONE)),
            Some(UiEvent::SqlInput('y'))
        );
        assert_eq!(
            map_key_event(key(KeyCode::Esc, KeyModifiers::NONE)),
            Some(UiEvent::CancelPending)
        );
        assert_eq!(
            map_key_event(key(KeyCode::Char('y'), KeyModifiers::CONTROL)),
            Some(UiEvent::AcceptCompletion)
        );
    }

    #[test]
    fn ignores_non_press_events() {
        let mut event = key(KeyCode::F(5), KeyModifiers::NONE);
        event.kind = KeyEventKind::Release;
        assert_eq!(map_key_event(event), None);
    }
}
