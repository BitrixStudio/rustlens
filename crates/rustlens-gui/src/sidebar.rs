use eframe::egui;

use crate::{driver_label, helpers, profiles, theme, GuiApp, WizardStep};

impl GuiApp {
    pub(crate) fn draw_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Profiles");
            if theme::secondary_button(ui, "Reload").clicked() {
                self.refresh_profiles();
            }
        });

        ui.small(profiles::profiles_path_text());
        ui.add_space(8.0);

        if self.profiles.is_empty() {
            ui.label("No profiles found.");
            if theme::primary_button(ui, "Open setup wizard").clicked() {
                self.wizard.visible = true;
                self.wizard.step = WizardStep::Welcome;
            }
        }

        let mut connect_index = None;
        let mut edit_index = None;
        let mut delete_index = None;
        for (idx, profile) in self.profiles.iter().enumerate() {
            let selected = self.selected_profile == Some(idx);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.selectable_label(selected, &profile.name).clicked() {
                        self.selected_profile = Some(idx);
                    }
                    if ui
                        .add_enabled(profile.supports_browsing(), egui::Button::new("Connect"))
                        .clicked()
                    {
                        connect_index = Some(idx);
                    }
                    if theme::secondary_button(ui, "Edit").clicked() {
                        edit_index = Some(idx);
                    }
                    if theme::danger_button(ui, "Delete").clicked() {
                        delete_index = Some(idx);
                    }
                });
                ui.small(format!(
                    "schema: {} | page size: {} | source: {:?}",
                    profile.schema, profile.page_size, profile.source
                ));
                ui.horizontal(|ui| {
                    theme::badge(ui, driver_label(profile.driver), theme::ACCENT);
                    if !profile.supports_browsing() {
                        theme::badge(ui, "planned", theme::WARNING);
                    }
                });
                if !profile.supports_browsing() {
                    ui.label(
                        egui::RichText::new(
                            "Browsing for this database type is planned but not implemented yet.",
                        )
                        .small()
                        .color(theme::MUTED),
                    );
                }
            });
        }

        if let Some(idx) = connect_index {
            self.connect_profile(idx);
        }
        if let Some(idx) = edit_index {
            self.pending_profile_edit = self.profiles.get(idx).map(helpers::edit_form_from_profile);
        }
        if let Some(idx) = delete_index {
            self.pending_profile_delete = self.profiles.get(idx).cloned();
        }

        ui.separator();
        ui.heading("Tables");
        if !self.connected {
            ui.label("Connect to a profile to load tables.");
            return;
        }

        let mut open_table = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for table in &self.tables {
                let selected = self.selected_table.as_ref() == Some(table);
                if ui.selectable_label(selected, table).clicked() {
                    open_table = Some(table.clone());
                }
            }
        });

        if let Some(table) = open_table {
            self.open_table(table);
        }
    }
}
