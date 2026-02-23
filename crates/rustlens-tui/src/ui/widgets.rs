use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};

use crate::app::state::{Focus, Tab};

pub fn top_bar(tab: Tab) -> Paragraph<'static> {
    let tab_name = match tab {
        Tab::Browse => "Browse",
        Tab::Sql => "SQL",
    };
    let help = "q quit  F2 Browse  F3 SQL  Tab focus  Enter open  PgUp/PgDn page  Ctrl+Enter run";
    Paragraph::new(Line::from(vec![
        Span::styled("rustlens", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("  |  Tab: {}  |  {}", tab_name, help)),
    ]))
}

pub fn bottom_bar<'a>(status: &'a str) -> Paragraph<'a> {
    Paragraph::new(status).block(Block::default().borders(Borders::TOP))
}

pub fn split_main(area: Rect) -> [Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);
    [chunks[0], chunks[1]]
}

pub fn tables_list<'a>(tables: &'a [String], focus: Focus) -> List<'a> {
    let items: Vec<ListItem> = tables
        .iter()
        .map(|t| ListItem::new(Line::from(t.as_str())))
        .collect();

    let title = match focus {
        Focus::Tables => "Tables (focus)",
        _ => "Tables",
    };

    List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
}

pub fn results_table<'a>(
    columns: &'a [String],
    rows: &'a [Vec<String>],
    focus: Focus,
    title: String,
) -> Table<'a> {
    let title = match focus {
        Focus::Results => format!("{} (focus)", title),
        _ => title,
    };

    let header =
        Row::new(columns.iter().cloned()).style(Style::default().add_modifier(Modifier::BOLD));
    let body: Vec<Row> = rows.iter().map(|r| Row::new(r.iter().cloned())).collect();

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
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
}

pub fn sql_editor<'a>(sql_text: &'a str, focus: Focus) -> Paragraph<'a> {
    let title = match focus {
        Focus::SqlEditor => "SQL Editor (focus)",
        _ => "SQL Editor",
    };

    Paragraph::new(sql_text).block(Block::default().borders(Borders::ALL).title(title))
}
