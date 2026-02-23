use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::Duration;

use crate::app::actions::{NavDir, PageDir};

#[derive(Debug)]
pub enum UiEvent {
    Quit,

    SwitchTabBrowse,
    SwitchTabSql,
    ToggleFocus,

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
}

pub fn poll_next_event(tick: Duration) -> Result<Option<UiEvent>> {
    if !event::poll(tick)? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(k) if k.kind == KeyEventKind::Press => {
            let ev = match (k.code, k.modifiers) {
                (KeyCode::Char('q'), _) => UiEvent::Quit,

                (KeyCode::F(2), _) => UiEvent::SwitchTabBrowse,
                (KeyCode::F(3), _) => UiEvent::SwitchTabSql,

                (KeyCode::Tab, _) => UiEvent::ToggleFocus,

                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => UiEvent::Nav(NavDir::Up),
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => UiEvent::Nav(NavDir::Down),

                (KeyCode::PageUp, _) => UiEvent::Page(PageDir::Prev),
                (KeyCode::PageDown, _) => UiEvent::Page(PageDir::Next),

                (KeyCode::F(5), KeyModifiers::CONTROL) => UiEvent::ExecuteSql,
                (KeyCode::Enter, KeyModifiers::CONTROL) => UiEvent::ExecuteSql,
                (KeyCode::Enter, KeyModifiers::NONE) => UiEvent::OpenSelection,

                (KeyCode::Char('r'), KeyModifiers::CONTROL) => UiEvent::Refresh,

                // SQL editing primitives
                (KeyCode::Backspace, _) => UiEvent::SqlBackspace,
                (KeyCode::Left, _) => UiEvent::SqlMoveCursorLeft,
                (KeyCode::Right, _) => UiEvent::SqlMoveCursorRight,
                (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => UiEvent::SqlInput(c),
                (KeyCode::Enter, _) => UiEvent::SqlNewline,

                _ => return Ok(None),
            };
            Ok(Some(ev))
        }
        _ => Ok(None),
    }
}
