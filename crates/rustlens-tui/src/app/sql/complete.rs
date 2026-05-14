use crate::app::state::SessionState;

pub fn refresh_completion(s: &mut SessionState) {
    let result = rustlens_core::sql::completion::complete(
        &s.sql_text,
        s.sql_cursor,
        &s.sql_tables,
        &s.sql_columns,
    );

    if !result.visible {
        s.completion.visible = false;
        s.completion.items.clear();
        s.completion.selected = 0;
        return;
    }

    s.completion.prefix_start = result.prefix_start;
    s.completion.items = result.items.into_iter().map(|item| item.label).collect();
    s.completion.selected = 0;
    s.completion.visible = true;
}

pub fn accept_completion(s: &mut SessionState) {
    if !s.completion.visible || s.completion.items.is_empty() {
        return;
    }

    let result = rustlens_core::sql::completion::complete(
        &s.sql_text,
        s.sql_cursor,
        &s.sql_tables,
        &s.sql_columns,
    );
    if !result.visible {
        s.completion.visible = false;
        return;
    }

    rustlens_core::sql::completion::apply_completion(
        &mut s.sql_text,
        &mut s.sql_cursor,
        &result,
        s.completion.selected,
    );
    s.completion.visible = false;
}
