use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::state::RootState;
use crate::ui::theme::Theme;

pub fn draw(f: &mut Frame, _root: &mut RootState, area: ratatui::layout::Rect, theme: &Theme) {
    let w = Paragraph::new("Manager mode (not implemented yet). Press F2 to switch to viewer.")
        .style(theme.text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_normal)
                .title("rustlensmanager"),
        );
    f.render_widget(w, area);
}
