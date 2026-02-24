use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Tabs},
};

use crate::app::state::{Focus, Tab};
use crate::ui::theme::Theme;

pub fn top_tabs(tab: Tab, theme: &Theme) -> Tabs<'static> {
    let titles = vec![
        Line::from(Span::styled(" [F2] Browse ", theme.tab_inactive)),
        Line::from(Span::styled(" [F3] SQL ", theme.tab_inactive)),
    ];

    let selected = match tab {
        Tab::Browse => 0,
        Tab::Sql => 1,
    };

    Tabs::new(titles)
        .select(selected)
        .style(theme.bar)
        .highlight_style(theme.tab_active)
}

pub fn theme_button(theme: &Theme) -> Paragraph<'static> {
    let label = "[Ctrl+T] Toggle Theme";
    let line = Line::from(vec![Span::styled(
        label,
        theme.tab_active.add_modifier(Modifier::BOLD),
    )]);

    Paragraph::new(line).style(theme.bar)
}

pub fn bottom_bar(width: u16, left: &str, right: &str, theme: &Theme) -> Paragraph<'static> {
    let w = width as usize;
    let left_len = left.chars().count();
    let right_len = right.chars().count();

    let spacer = if w > left_len + right_len + 1 {
        " ".repeat(w - left_len - right_len)
    } else {
        " ".into()
    };

    let line = Line::from(vec![
        Span::styled(left.to_string(), theme.status_left),
        Span::raw(spacer),
        Span::styled(right.to_string(), theme.status_right),
    ]);

    Paragraph::new(line).style(theme.bar)
}

pub fn split_main(area: Rect) -> [Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);
    [chunks[0], chunks[1]]
}

fn block_with_border(title: String, focused: bool, theme: &Theme) -> Block<'static> {
    let border_style = if focused {
        theme.border_focused
    } else {
        theme.border_normal
    };

    Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            title,
            if focused { theme.text } else { theme.muted },
        ))
}

pub fn tables_list<'a>(tables: &'a [String], focus: Focus, theme: &Theme) -> List<'a> {
    let items: Vec<ListItem> = tables
        .iter()
        .map(|t| ListItem::new(Line::from(Span::styled(t.as_str(), theme.list_item))))
        .collect();

    let focused = matches!(focus, Focus::Tables);
    let title = if focused {
        "Tables (focus)".to_string()
    } else {
        "Tables".to_string()
    };

    List::new(items)
        .block(block_with_border(title, focused, theme))
        .style(theme.text)
        .highlight_style(theme.list_item_selected)
}

pub fn results_table<'a>(
    columns: &'a [String],
    rows: &'a [Vec<String>],
    focus: Focus,
    title: String,
    theme: &Theme,
) -> Table<'a> {
    let focused = matches!(focus, Focus::Results);
    let title = if focused {
        format!("{} (focus)", title)
    } else {
        title
    };

    let header = Row::new(columns.iter().cloned()).style(theme.table_header);

    let body: Vec<Row> = rows
        .iter()
        .map(|r| Row::new(r.iter().cloned()).style(theme.table_row))
        .collect();

    let widths = if columns.is_empty() {
        vec![Constraint::Min(1)]
    } else {
        columns
            .iter()
            .map(|_| Constraint::Ratio(1, columns.len() as u32))
            .collect()
    };

    Table::new(body, widths)
        .header(header)
        .block(block_with_border(title, focused, theme))
        .style(theme.text)
        .row_highlight_style(theme.table_row_selected)
}

pub fn sql_editor<'a>(sql_text: &'a str, focus: Focus, theme: &Theme) -> Paragraph<'a> {
    let focused = matches!(focus, Focus::SqlEditor);
    let title = if focused {
        "SQL Editor (focus)".to_string()
    } else {
        "SQL Editor".to_string()
    };

    Paragraph::new(sql_text)
        .style(theme.editor_text)
        .block(block_with_border(title, focused, theme))
}
