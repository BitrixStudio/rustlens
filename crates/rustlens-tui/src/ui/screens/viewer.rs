use ratatui::prelude::Rect;
use ratatui::Frame;

use crate::app::state::RootState;
use crate::ui::theme::Theme;
use crate::ui::widgets;

pub fn draw(f: &mut Frame, root: &mut RootState, area: Rect, theme: &Theme) {
    let s = &mut root.session;

    match s.tab {
        crate::app::state::Tab::Browse => {
            let [left, right] = widgets::split_main(area);

            let list = widgets::tables_list(&s.tables, s.focus, theme);
            f.render_stateful_widget(list, left, &mut s.tables_state);

            let title = match &s.selected_table {
                Some(t) => format!("Results: {} | page {}", t, s.page + 1),
                None => "Results".to_string(),
            };

            let table = widgets::results_table(&s.columns, &s.rows, s.focus, title, theme);
            f.render_stateful_widget(table, right, &mut s.results_state);
        }

        crate::app::state::Tab::Sql => {
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Percentage(40),
                    ratatui::layout::Constraint::Percentage(60),
                ])
                .split(area);

            let highlighted = root.sql_syntax.highlight(&s.sql_text);
            f.render_widget(widgets::sql_editor(highlighted, s.focus, theme), chunks[0]);

            let title = match &s.selected_table {
                Some(t) => format!("Results: {} | page {}", t, s.page + 1),
                None => "Results".to_string(),
            };

            let (line, col) = crate::app::sql::cursor::cursor_line_col(&s.sql_text, s.sql_cursor);
            let table = widgets::results_table(&s.columns, &s.rows, s.focus, title, theme);
            f.render_stateful_widget(table, chunks[1], &mut s.results_state);

            if s.completion_enabled && s.completion.visible && !s.completion.items.is_empty() {
                let visible_items = s.completion.items.len().min(8);
                let popup_h = (visible_items as u16) + 2;
                let popup_w = 32;

                let popup = popup_rect_near_cursor(chunks[0], line, col, popup_h, popup_w);

                let items: Vec<ratatui::widgets::ListItem> = s
                    .completion
                    .items
                    .iter()
                    .take(visible_items)
                    .map(|k| ratatui::widgets::ListItem::new(*k))
                    .collect();

                let mut state = ratatui::widgets::ListState::default();
                state.select(Some(
                    s.completion.selected.min(visible_items.saturating_sub(1)),
                ));

                let list = ratatui::widgets::List::new(items)
                    .block(
                        ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                            .title("Complete"),
                    )
                    .highlight_style(theme.list_item_selected)
                    .highlight_symbol("▶ ");

                f.render_stateful_widget(list, popup, &mut state);
            }
        }
    }

    fn clamp_u16(v: i32) -> u16 {
        if v < 0 {
            0
        } else {
            v as u16
        }
    }

    fn popup_rect_near_cursor(
        editor: Rect,
        line: usize,
        col: usize,
        height: u16,
        width: u16,
    ) -> Rect {
        // Here add: +1,+1 to leave place for borders inside the block
        // show popup 1 line below cursor
        // This code also looks like hacky workaround, investigate if it could be further optimized
        let mut x = editor.x as i32 + 1 + col as i32;
        let mut y = editor.y as i32 + 1 + line as i32 + 1;

        let max_x = (editor.x + editor.width).saturating_sub(1) as i32;
        let max_y = (editor.y + editor.height).saturating_sub(1) as i32;

        if x + width as i32 > max_x {
            x = max_x - width as i32;
        }
        if y + height as i32 > max_y {
            // if not enough space below, try above cursor
            y = (editor.y as i32 + 1 + line as i32) - height as i32;
        }
        if y < editor.y as i32 + 1 {
            y = editor.y as i32 + 1;
        }

        Rect {
            x: clamp_u16(x),
            y: clamp_u16(y),
            width: width.min(editor.width.saturating_sub(2)).max(10),
            height: height.min(editor.height.saturating_sub(2)).max(3),
        }
    }
}
