use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Tabs},
};

use crate::app::state::{Focus, Tab};
use crate::ui::theme::Theme;

pub enum BottomBarMode {
    MiddleCentered,
    MiddleAndRightRightAligned,
}

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

fn take_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

pub fn bottom_bar(
    width: u16,
    left: &str,
    middle: &str,
    right: &str,
    mode: BottomBarMode,
    theme: &Theme,
) -> Paragraph<'static> {
    let w = width as usize;

    // Early exit: nothing fits
    if w == 0 {
        return Paragraph::new(Line::from(Vec::<Span>::new())).style(theme.bar);
    }

    let mut spans: Vec<Span> = Vec::new();
    match mode {
        // This whole section looks like a hack/workaround
        // and should be thought out further after MVP phase
        BottomBarMode::MiddleAndRightRightAligned => {
            spans.push(Span::styled(left.to_string(), theme.status_left));

            let w = w;
            let left_len = left.chars().count();

            if left_len >= w {
                return Paragraph::new(Line::from(spans)).style(theme.bar);
            }

            let remaining = w - left_len;

            let middle_present = !middle.is_empty();
            let mut middle_piece = if middle_present {
                format!(" {} ", middle)
            } else {
                String::new()
            };

            let sep = if middle_present { " " } else { "" };
            let mut right_piece = right.to_string();

            // We will fit into remaining by truncating middle first, then right
            loop {
                let right_len = right_piece.chars().count();
                let middle_len = middle_piece.chars().count();
                let sep_len = sep.chars().count();

                // We are aiming for: [spacer][middle][sep][right]
                let group_len = middle_len + sep_len + right_len;

                if group_len <= remaining {
                    // spacer goes between left and group, pushing group to the right edge
                    let spacer_len = remaining - group_len;
                    spans.push(Span::raw(" ".repeat(spacer_len)));

                    if middle_present && !middle_piece.is_empty() {
                        spans.push(Span::styled(middle_piece.clone(), theme.status_middle));
                        if !sep.is_empty() {
                            spans.push(Span::raw(sep));
                        }
                    }

                    spans.push(Span::styled(right_piece.clone(), theme.status_right));
                    break;
                }

                // Too long -> truncate middle first
                if middle_present && !middle_piece.is_empty() {
                    // we need to leave room for sep + right
                    let right_len = right_piece.chars().count();
                    let sep_len = sep.chars().count();
                    let max_for_middle = remaining.saturating_sub(sep_len + right_len);

                    if max_for_middle >= 2 {
                        // keep padding "  " around inner text
                        let inner_max = max_for_middle - 2;
                        let inner = take_chars(middle, inner_max);
                        middle_piece = format!(" {} ", inner);
                    } else {
                        // not enough space even for padded middle -> drop it
                        middle_piece.clear();
                    }
                    continue;
                }

                // No middle text, still too long -> truncate right
                let right_max = remaining; // only right remains (sep is empty if no middle)
                right_piece = take_chars(&right_piece, right_max);
                // next loop will fit
            }
        }

        BottomBarMode::MiddleCentered => {
            spans.push(Span::styled(take_chars(left, w), theme.status_left));
        }
    }
    Paragraph::new(Line::from(spans)).style(theme.bar)
}

pub fn split_main(area: Rect) -> [Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);
    [chunks[0], chunks[1]]
}

pub fn block_with_border(title: Line<'static>, focused: bool, theme: &Theme) -> Block<'static> {
    let border_style = if focused {
        theme.border_focused
    } else {
        theme.border_normal
    };

    Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .style(theme.text)
}

fn title_pill(title: &str, focused: bool, theme: &Theme) -> Line<'static> {
    let style = if focused {
        theme.tab_active
    } else {
        theme.tab_inactive
    };

    Line::from(vec![Span::styled(format!(" {title} "), style)])
}

pub fn tables_list<'a>(tables: &'a [String], focus: Focus, theme: &Theme) -> List<'a> {
    let items: Vec<ListItem> = tables
        .iter()
        .map(|t| ListItem::new(Line::from(Span::styled(t.as_str(), theme.list_item))))
        .collect();

    let focused = matches!(focus, Focus::Tables);
    let title = title_pill("Tables", focused, theme);

    List::new(items)
        .block(block_with_border(title, focused, theme))
        .style(theme.text)
        .highlight_style(theme.list_item_selected)
        .highlight_symbol("▶ ")
}

pub fn results_table<'a>(
    columns: &'a [String],
    rows: &'a [Vec<String>],
    focus: Focus,
    title: String,
    theme: &Theme,
) -> Table<'a> {
    let focused = matches!(focus, Focus::Results);
    let title = title_pill(&title, focused, theme);

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

pub fn sql_editor(
    highlighted: &[Line<'static>],
    focus: Focus,
    theme: &Theme,
) -> Paragraph<'static> {
    let focused = matches!(focus, Focus::SqlEditor);
    let title = title_pill("SQL", focused, theme);

    Paragraph::new(Text::from(highlighted.to_vec())).block(block_with_border(title, focused, theme))
}
