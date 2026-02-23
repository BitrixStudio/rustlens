use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct UiRects {
    pub top: Rect,
    pub main: Rect,
    pub bottom: Rect,
}

pub fn split_root(area: Rect) -> UiRects {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    UiRects {
        top: chunks[0],
        main: chunks[1],
        bottom: chunks[2],
    }
}
