use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::state::RootState;

pub fn draw(f: &mut Frame, _root: &mut RootState, area: ratatui::layout::Rect) {
    let w = Paragraph::new("Manager mode (not implemented yet). Press F2 to switch to viewer.")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("rustlensmanager"),
        );
    f.render_widget(w, area);
}
