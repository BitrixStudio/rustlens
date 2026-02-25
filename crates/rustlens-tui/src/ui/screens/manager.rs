use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::state::RootState;
use crate::ui::theme::Theme;

pub fn draw(f: &mut Frame, root: &mut RootState, area: Rect, theme: &Theme) {
    // Clear the entire manager area so old viewer frames don't remain.
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(area);

    let items: Vec<ListItem> = root
        .manager
        .profiles
        .iter()
        .map(|p| ListItem::new(p.name.clone()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Profiles"))
        .highlight_style(theme.list_item_selected)
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, chunks[0], &mut root.manager.list_state);

    let details = if let Some(p) = root.manager.selected() {
        format!(
            "Name: {}\nSchema: {}\n\nURL:\n{}\n\nEnter: open\nq: quit",
            p.name, p.schema, p.database_url
        )
    } else {
        "No profiles.\n\nEnter: open\nq: quit".into()
    };

    let right = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .style(theme.text);

    f.render_widget(right, chunks[1]);
}
