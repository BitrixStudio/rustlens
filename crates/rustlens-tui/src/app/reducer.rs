use tokio::sync::mpsc;

use rustlens_core::db;

use crate::app::actions::{NavDir, PageDir};
use crate::app::event::AppEvent;
use crate::app::state::{Focus, RootState, Tab};
use crate::term::input::UiEvent;

pub async fn reduce(
    root: &mut RootState,
    ev: AppEvent,
    db_cmd_tx: &mpsc::Sender<db::DbCmd>,
) -> bool {
    match ev {
        AppEvent::Db(evt) => {
            handle_db(root, evt, db_cmd_tx).await;
            false
        }
        AppEvent::Input(evt) => handle_input(root, evt, db_cmd_tx).await,
    }
}

async fn handle_db(
    root: &mut RootState,
    evt: db::DbEvt,
    db_cmd_tx: &mpsc::Sender<db::DbCmd>,
) {
    match evt {
        db::DbEvt::Status(msg) => root.status = msg,

        db::DbEvt::Error(e) => root.status = format!("Error: {e}"),

        db::DbEvt::TablesLoaded { tables } => {
            root.session.tables = tables;
            root.session.tables_state.select(Some(0));

            if root.session.tables.is_empty() {
                root.status = format!("No tables found in schema '{}'.", root.session.schema);
            } else {
                root.status = "Tables loaded. Enter to open.".into();
            }
        }

        db::DbEvt::QueryResult { columns, rows, info } => {
            root.session.columns = columns;
            root.session.rows = rows;
            root.session.results_state.select(Some(0));
            root.status = info;
        }

        db::DbEvt::SqlExecuted { info } => {
            root.session.sql_last_result = Some(info.clone());
            root.status = info;
        }
    }

    // Auto-open first table once tables arrive.
    if root.session.selected_table.is_none() && !root.session.tables.is_empty() {
        if let Some(t) = root.session.selected_table_from_list().map(|x| x.to_string()) {
            root.session.selected_table = Some(t.clone());
            root.session.page = 0;

            let schema = root.session.schema.clone();
            let page_size = root.session.page_size;

            let _ = db_cmd_tx
                .send(db::DbCmd::LoadTablePage {
                    schema,
                    table: t,
                    page: 0,
                    page_size,
                })
                .await;
        }
    }
}

async fn handle_input(
    root: &mut RootState,
    ev: UiEvent,
    db_cmd_tx: &mpsc::Sender<db::DbCmd>,
) -> bool {
    use UiEvent::*;

    let s = &mut root.session;

    match ev {
        Quit => return true,

        SwitchTabBrowse => {
            s.tab = Tab::Browse;
            s.focus = Focus::Tables;
            root.status = "Browse".into();
        }
        SwitchTabSql => {
            s.tab = Tab::Sql;
            s.focus = Focus::SqlEditor;
            root.status = "SQL".into();
        }

        ToggleFocus => toggle_focus(s),

        Nav(dir) => match s.focus {
            Focus::Tables => nav_list(&mut s.tables_state, s.tables.len(), dir),
            Focus::Results => nav_table(&mut s.results_state, s.rows.len(), dir),
            Focus::SqlEditor => {}
        },

        Page(dir) => {
            if s.tab == Tab::Browse {
                let table = s
                    .selected_table
                    .clone()
                    .or_else(|| s.selected_table_from_list().map(|x| x.to_string()));

                if let Some(table) = table {
                    match dir {
                        PageDir::Next => s.page += 1,
                        PageDir::Prev => s.page = (s.page - 1).max(0),
                    }
                    let _ = db_cmd_tx
                        .send(db::DbCmd::LoadTablePage {
                            schema: s.schema.clone(),
                            table,
                            page: s.page,
                            page_size: s.page_size,
                        })
                        .await;
                }
            }
        }

        OpenSelection => {
            if s.tab == Tab::Browse {
                if let Some(table) = s.selected_table_from_list().map(|x| x.to_string()) {
                    s.selected_table = Some(table.clone());
                    s.page = 0;
                    let _ = db_cmd_tx
                        .send(db::DbCmd::LoadTablePage {
                            schema: s.schema.clone(),
                            table,
                            page: 0,
                            page_size: s.page_size,
                        })
                        .await;
                }
            } else {
                // Enter in SQL editor inserts newline. Use F5/Ctrl+R to execute.
                s.sql_text.insert(s.sql_cursor, '\n');
                s.sql_cursor += 1;
            }
        }

        Refresh => {
            let _ = db_cmd_tx
                .send(db::DbCmd::LoadTables {
                    schema: s.schema.clone(),
                })
                .await;
        }

        SqlInput(ch) => {
            if s.focus == Focus::SqlEditor {
                s.sql_text.insert(s.sql_cursor, ch);
                s.sql_cursor += ch.len_utf8();
            }
        }
        SqlBackspace => {
            if s.focus == Focus::SqlEditor && s.sql_cursor > 0 {
                let prev = s.sql_text[..s.sql_cursor]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                s.sql_text.drain(prev..s.sql_cursor);
                s.sql_cursor = prev;
            }
        }
        SqlNewline => {
            if s.focus == Focus::SqlEditor {
                s.sql_text.insert(s.sql_cursor, '\n');
                s.sql_cursor += 1;
            }
        }
        SqlMoveCursorLeft => {
            if s.focus == Focus::SqlEditor && s.sql_cursor > 0 {
                let prev = s.sql_text[..s.sql_cursor]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                s.sql_cursor = prev;
            }
        }
        SqlMoveCursorRight => {
            if s.focus == Focus::SqlEditor && s.sql_cursor < s.sql_text.len() {
                let mut it = s.sql_text[s.sql_cursor..].char_indices();
                if let Some((i, ch)) = it.next() {
                    s.sql_cursor += i + ch.len_utf8();
                }
            }
        }

        ExecuteSql => {
            if s.tab == Tab::Sql {
                let sql = s.sql_text.trim().to_string();
                if sql.is_empty() {
                    root.status = "SQL is empty.".into();
                } else {
                    let _ = db_cmd_tx.send(db::DbCmd::ExecuteSql { sql }).await;
                }
            }
        }
    }

    false
}

fn toggle_focus(s: &mut crate::app::state::SessionState) {
    use Focus::*;
    use Tab::*;

    s.focus = match (s.tab, s.focus) {
        (Browse, Tables) => Results,
        (Browse, Results) => Tables,
        (Browse, SqlEditor) => Tables,

        (Sql, SqlEditor) => Results,
        (Sql, Results) => SqlEditor,
        (Sql, Tables) => SqlEditor,
    };
}

fn nav_list(state: &mut ratatui::widgets::ListState, len: usize, dir: NavDir) {
    if len == 0 {
        return;
    }
    let i = state.selected().unwrap_or(0);
    let ni = match dir {
        NavDir::Up => i.saturating_sub(1),
        NavDir::Down => (i + 1).min(len - 1),
    };
    state.select(Some(ni));
}

fn nav_table(state: &mut ratatui::widgets::TableState, len: usize, dir: NavDir) {
    if len == 0 {
        return;
    }
    let i = state.selected().unwrap_or(0);
    let ni = match dir {
        NavDir::Up => i.saturating_sub(1),
        NavDir::Down => (i + 1).min(len - 1),
    };
    state.select(Some(ni));
}
