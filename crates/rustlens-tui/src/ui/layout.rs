use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct RootLayout {
    pub top: Rect,
    pub main: Rect,
    pub bottom: Rect,
}

pub fn split_root(area: Rect) -> RootLayout {
    // Ensure we always reserve top+bottom bars.
    // If the terminal is too small, fall back gracefully.
    if area.height < 3 {
        return RootLayout {
            top: area,
            main: Rect { x: area.x, y: area.y, width: area.width, height: 0 },
            bottom: Rect { x: area.x, y: area.y, width: area.width, height: 0 },
        };
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top bar
            Constraint::Min(1),    // main
            Constraint::Length(1), // bottom bar
        ])
        .split(area);

    RootLayout {
        top: chunks[0],
        main: chunks[1],
        bottom: chunks[2],
    }
}
