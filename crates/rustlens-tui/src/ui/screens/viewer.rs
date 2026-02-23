use ratatui::Frame;

use crate::app::state::RootState;
use crate::ui::widgets;

pub fn draw(f: &mut Frame, root: &mut RootState, area: ratatui::layout::Rect) {
    let s = &mut root.session;

    match s.tab {
        crate::app::state::Tab::Browse => {
            let [left, right] = widgets::split_main(area);

            let list = widgets::tables_list(&s.tables, s.focus);
            f.render_stateful_widget(list, left, &mut s.tables_state);

            let title = match &s.selected_table {
                Some(t) => format!("Results: {} | page {}", t, s.page + 1),
                None => "Results".to_string(),
            };
            let table = widgets::results_table(&s.columns, &s.rows, s.focus, title);
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

            f.render_widget(widgets::sql_editor(&s.sql_text, s.focus), chunks[0]);

            let title = s
                .sql_last_result
                .clone()
                .unwrap_or_else(|| "SQL Results".into());

            let table = widgets::results_table(&s.columns, &s.rows, s.focus, title);
            f.render_stateful_widget(table, chunks[1], &mut s.results_state);
        }
    }
}
