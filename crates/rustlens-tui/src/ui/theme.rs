use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    Default,
    SolarizedDark,
    GruvboxDark,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub bar: Style,
    pub tab_active: Style,
    pub tab_inactive: Style,

    pub border_focused: Style,
    pub border_normal: Style,

    pub text: Style,
    pub muted: Style,

    pub table_header: Style,
    pub table_row: Style,
    pub table_row_selected: Style,

    pub list_item: Style,
    pub list_item_selected: Style,

    pub editor_text: Style,
    pub editor_cursor: Style,

    pub status_left: Style,
    pub status_right: Style,
}

impl Theme {
    pub fn from_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Default => Self {
                bar: Style::default().fg(Color::White).bg(Color::Black),
                tab_active: Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                tab_inactive: Style::default().fg(Color::Gray),

                border_focused: Style::default().fg(Color::Cyan),
                border_normal: Style::default().fg(Color::DarkGray),

                text: Style::default().fg(Color::White),
                muted: Style::default().fg(Color::Gray),

                table_header: Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
                table_row: Style::default().fg(Color::White),
                table_row_selected: Style::default().fg(Color::Black).bg(Color::Cyan),

                list_item: Style::default().fg(Color::White),
                list_item_selected: Style::default().fg(Color::Black).bg(Color::Cyan),

                editor_text: Style::default().fg(Color::White),
                editor_cursor: Style::default().bg(Color::White).fg(Color::Black),

                status_left: Style::default().fg(Color::White),
                status_right: Style::default().fg(Color::Gray),
            },

            ThemeKind::SolarizedDark => {
                // Solarized Dark-ish (using closest built-in ANSI colors)
                Self {
                    bar: Style::default().fg(Color::Yellow).bg(Color::Black),
                    tab_active: Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    tab_inactive: Style::default().fg(Color::DarkGray),

                    border_focused: Style::default().fg(Color::Yellow),
                    border_normal: Style::default().fg(Color::DarkGray),

                    text: Style::default().fg(Color::White),
                    muted: Style::default().fg(Color::Gray),

                    table_header: Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    table_row: Style::default().fg(Color::White),
                    table_row_selected: Style::default().fg(Color::Black).bg(Color::Yellow),

                    list_item: Style::default().fg(Color::White),
                    list_item_selected: Style::default().fg(Color::Black).bg(Color::Yellow),

                    editor_text: Style::default().fg(Color::White),
                    editor_cursor: Style::default().bg(Color::Yellow).fg(Color::Black),

                    status_left: Style::default().fg(Color::Yellow),
                    status_right: Style::default().fg(Color::DarkGray),
                }
            }

            ThemeKind::GruvboxDark => Self {
                bar: Style::default().fg(Color::LightRed).bg(Color::Black),
                tab_active: Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
                tab_inactive: Style::default().fg(Color::Gray),

                border_focused: Style::default().fg(Color::LightRed),
                border_normal: Style::default().fg(Color::DarkGray),

                text: Style::default().fg(Color::White),
                muted: Style::default().fg(Color::Gray),

                table_header: Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
                table_row: Style::default().fg(Color::White),
                table_row_selected: Style::default().fg(Color::Black).bg(Color::LightRed),

                list_item: Style::default().fg(Color::White),
                list_item_selected: Style::default().fg(Color::Black).bg(Color::LightRed),

                editor_text: Style::default().fg(Color::White),
                editor_cursor: Style::default().bg(Color::LightRed).fg(Color::Black),

                status_left: Style::default().fg(Color::LightRed),
                status_right: Style::default().fg(Color::Gray),
            },
        }
    }
}
