use eframe::egui;

pub const BG: egui::Color32 = egui::Color32::from_rgb(12, 17, 23);
pub const PANEL: egui::Color32 = egui::Color32::from_rgb(18, 25, 33);
pub const PANEL_ALT: egui::Color32 = egui::Color32::from_rgb(25, 34, 45);
pub const PRIMARY: egui::Color32 = egui::Color32::from_rgb(45, 212, 191);
pub const PRIMARY_DARK: egui::Color32 = egui::Color32::from_rgb(17, 94, 89);
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(96, 165, 250);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(245, 158, 11);
pub const DANGER: egui::Color32 = egui::Color32::from_rgb(239, 68, 68);
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(229, 231, 235);
pub const MUTED: egui::Color32 = egui::Color32::from_rgb(148, 163, 184);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(51, 65, 85);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = BG;
    visuals.faint_bg_color = PANEL_ALT;
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = PRIMARY_DARK;
    visuals.selection.stroke = egui::Stroke::new(1.0, PRIMARY);
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT);
    visuals.widgets.inactive.bg_fill = PANEL_ALT;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(34, 48, 64);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT);
    visuals.widgets.active.bg_fill = PRIMARY_DARK;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT);
    visuals.window_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, PRIMARY);
    visuals.window_rounding = egui::Rounding::same(10.0);
    visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
    visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
    visuals.widgets.active.rounding = egui::Rounding::same(8.0);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    ctx.set_style(style);
}

pub fn top_bar_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(9, 14, 20))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn side_panel_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(PANEL)
        .inner_margin(egui::Margin::symmetric(10.0, 10.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn status_bar_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(9, 14, 20))
        .inner_margin(egui::Margin::symmetric(12.0, 7.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(primary_button_widget(label))
}

pub fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(secondary_button_widget(label))
}

pub fn danger_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(danger_button_widget(label))
}

pub fn primary_button_widget(label: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(label).strong().color(BG)).fill(PRIMARY)
}

pub fn secondary_button_widget(label: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(label).color(TEXT)).fill(PANEL_ALT)
}

pub fn danger_button_widget(label: &str) -> egui::Button<'_> {
    egui::Button::new(
        egui::RichText::new(label)
            .strong()
            .color(egui::Color32::WHITE),
    )
    .fill(DANGER)
}

pub fn badge(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    egui::Frame::none()
        .fill(color.gamma_multiply(0.22))
        .rounding(egui::Rounding::same(999.0))
        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).small().color(color));
        });
}
