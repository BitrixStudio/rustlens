use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::state::{Mode, RootState};
use crate::ui::widgets::BottomBarMode;
use crate::ui::{layout, widgets};

pub fn draw(f: &mut Frame, root: &mut RootState) {
    let rects = layout::split_root(f.area());

    // Take a snapshot of the theme for this frame to avoid borrow conflicts
    // Potentially not a great solution but a quick workaround made
    // by my ever so expanding but limited Rust knowledge
    let theme = root.theme.clone();

    // Split top bar into tabs (left) and theme button (right)
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(22)])
        .split(rects.top);

    f.render_widget(widgets::top_tabs(root.session.tab, &theme), top_chunks[0]);
    f.render_widget(widgets::theme_button(&theme), top_chunks[1]);

    match root.mode {
        Mode::Viewer => crate::ui::screens::viewer::draw(f, root, rects.main, &theme),
        Mode::Manager => crate::ui::screens::manager::draw(f, root, rects.main, &theme),
    }

    f.render_widget(
        widgets::bottom_bar(
            rects.bottom.width,
            &root.status.left,
            &root.status.middle,
            &root.status.right,
            BottomBarMode::MiddleAndRightRightAligned,
            &theme,
        ),
        rects.bottom,
    );
}
