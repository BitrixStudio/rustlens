use ratatui::Frame;

use crate::app::state::{Mode, RootState};
use crate::ui::{layout, widgets};

pub fn draw(f: &mut Frame, root: &mut RootState) {
    let rects = layout::split_root(f.area());

    let tab = root.session.tab;
    f.render_widget(widgets::top_bar(tab), rects.top);

    match root.mode {
        Mode::Viewer => crate::ui::screens::viewer::draw(f, root, rects.main),
        Mode::Manager => crate::ui::screens::manager::draw(f, root, rects.main),
    }

    f.render_widget(
        widgets::bottom_bar_line(rects.bottom.width, &root.status.left, &root.status.right),
        rects.bottom,
    );
}
