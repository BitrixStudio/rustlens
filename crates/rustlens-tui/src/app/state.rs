use crate::ui::syntax::SqlSyntax;
use crate::ui::theme::{Theme, ThemeKind};
use crate::{config::AppConfig, LaunchMode};
use ratatui::widgets::{ListState, TableState};
use std::time::Duration;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Browse,
    Sql,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tables,
    Results,
    SqlEditor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Viewer,
    Manager,
}

#[derive(Debug, Default, Clone)]
pub struct StatusBar {
    pub left: String,
    pub right: String,
    pub middle: String,
}

#[derive(Debug)]
pub struct RootState {
    pub mode: Mode,
    pub status: StatusBar,
    pub session: SessionState,
    pub sql_syntax: SqlSyntax,

    pub theme_kind: ThemeKind,
    pub theme: Theme,
}

#[derive(Clone, Debug)]
pub struct CompletionState {
    pub items: Vec<&'static str>,
    pub selected: usize,
    pub visible: bool,
    pub prefix_start: usize,
}

impl Default for CompletionState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            visible: false,
            prefix_start: 0,
        }
    }
}

#[derive(Debug)]
pub struct SessionState {
    pub tick_rate: Duration,

    pub tab: Tab,
    pub focus: Focus,

    pub schema: String,
    pub page_size: i64,

    pub tables: Vec<String>,
    pub tables_state: ListState,

    pub selected_table: Option<String>,
    pub page: i64,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub results_state: TableState,
    pub completion: CompletionState,
    pub completion_enabled: bool,

    pub sql_text: String,
    pub sql_cursor: usize,
    pub sql_last_result: Option<String>,
}

impl RootState {
    pub fn new(cfg: AppConfig, launch: LaunchMode) -> Self {
        let mode = match launch {
            LaunchMode::Viewer { .. } => Mode::Viewer,
            LaunchMode::Manager => Mode::Manager,
        };
        let theme_kind = ThemeKind::Default;
        Self {
            mode,
            status: StatusBar {
                left: "Starting…".into(),
                right: String::new(),
                middle: String::new(),
            },
            session: SessionState::new(cfg),
            theme: Theme::from_kind(theme_kind),
            theme_kind,
            sql_syntax: crate::ui::syntax::SqlSyntax::new(),
        }
    }
    pub fn cycle_theme(&mut self) {
        use crate::ui::theme::ThemeKind::*;

        self.theme_kind = match self.theme_kind {
            Default => SolarizedDark,
            SolarizedDark => GruvboxDark,
            GruvboxDark => Default,
        };

        self.theme = crate::ui::theme::Theme::from_kind(self.theme_kind);
        self.status.middle = format!("Theme: {}", self.theme_kind.as_str());
    }
}

impl ThemeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ThemeKind::Default => "Default",
            ThemeKind::SolarizedDark => "Solarized Dark",
            ThemeKind::GruvboxDark => "Gruvbox Dark",
        }
    }
}

impl SessionState {
    pub fn new(cfg: AppConfig) -> Self {
        let mut tables_state = ListState::default();
        tables_state.select(Some(0));
        let mut results_state = TableState::default();
        results_state.select(Some(0));

        Self {
            tick_rate: Duration::from_millis(50),
            tab: Tab::Browse,
            focus: Focus::Tables,

            schema: cfg.schema,
            page_size: cfg.page_size,

            tables: vec![],
            tables_state,

            selected_table: None,
            page: 0,
            columns: vec![],
            rows: vec![],
            results_state,
            completion: CompletionState::default(),
            completion_enabled: true,
            sql_text: String::new(),
            sql_cursor: 0,
            sql_last_result: None,
        }
    }

    pub fn selected_table_from_list(&self) -> Option<&str> {
        self.tables_state
            .selected()
            .and_then(|i| self.tables.get(i))
            .map(|s| s.as_str())
    }
}
