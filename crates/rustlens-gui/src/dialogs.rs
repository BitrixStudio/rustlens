use eframe::egui;

use crate::{driver_label, grid_form, profiles, theme, GuiApp};

impl GuiApp {
    pub(crate) fn draw_confirmation(&mut self, ctx: &egui::Context) {
        if self.pending_sql_confirmation.is_none() {
            return;
        }

        let mut cancel = false;
        let mut confirm = false;
        let sql = self.pending_sql_confirmation.as_deref().unwrap_or_default();

        egui::Window::new("Confirm SQL Execution")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("This statement may modify data or schema.");
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        ui.monospace(sql);
                    });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "Cancel").clicked() {
                        cancel = true;
                    }
                    if theme::danger_button(ui, "Execute").clicked() {
                        confirm = true;
                    }
                });
            });

        if cancel {
            self.pending_sql_confirmation = None;
            self.detail_status = "SQL execution cancelled.".to_string();
        }
        if confirm {
            self.confirm_pending_sql();
        }
    }

    pub(crate) fn draw_schema_confirmation(&mut self, ctx: &egui::Context) {
        let Some(statements) = self.pending_schema_confirmation.as_ref() else {
            return;
        };

        let mut cancel = false;
        let mut confirm = false;
        let statement_count = statements.len();
        let sql = statements.join("\n\n");

        egui::Window::new("Confirm Schema Apply")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Apply {statement_count} schema statement(s) to schema '{}' ?",
                    self.schema
                ));
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .show(ui, |ui| {
                        ui.monospace(&sql);
                    });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "Cancel").clicked() {
                        cancel = true;
                    }
                    if theme::danger_button(ui, "Apply schema").clicked() {
                        confirm = true;
                    }
                });
            });

        if cancel {
            self.pending_schema_confirmation = None;
            self.detail_status = "Schema apply cancelled.".to_string();
        }
        if confirm {
            self.confirm_pending_schema();
        }
    }

    pub(crate) fn draw_profile_edit(&mut self, ctx: &egui::Context) {
        let Some(mut form) = self.pending_profile_edit.clone() else {
            return;
        };

        let mut cancel = false;
        let mut save = false;

        egui::Window::new("Edit Profile")
            .collapsible(false)
            .resizable(true)
            .default_width(620.0)
            .show(ctx, |ui| {
                ui.label(format!("Driver: {}", driver_label(form.driver)));
                ui.small(format!("Source: {:?}", form.source));
                if form.dbnest_instance_id.is_some() {
                    ui.colored_label(
                        theme::WARNING,
                        "Editing the RustLens profile does not modify the dbnest instance.",
                    );
                }
                ui.add_space(8.0);

                grid_form(ui, "edit_profile_grid", |ui| {
                    ui.label("Profile name");
                    ui.text_edit_singleline(&mut form.name);
                    ui.end_row();
                    ui.label("Database URL");
                    ui.text_edit_singleline(&mut form.database_url);
                    ui.end_row();
                    ui.label("Schema");
                    ui.text_edit_singleline(&mut form.schema);
                    ui.end_row();
                    ui.label("Page size");
                    ui.text_edit_singleline(&mut form.page_size);
                    ui.end_row();
                });

                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "Cancel").clicked() {
                        cancel = true;
                    }
                    if theme::primary_button(ui, "Save profile").clicked() {
                        save = true;
                    }
                });
            });

        if cancel {
            self.pending_profile_edit = None;
            self.detail_status = "Profile edit cancelled.".to_string();
        } else {
            self.pending_profile_edit = Some(form);
            if save {
                self.save_profile_edit();
            }
        }
    }

    pub(crate) fn draw_profile_delete_confirmation(&mut self, ctx: &egui::Context) {
        let Some(profile) = self.pending_profile_delete.clone() else {
            return;
        };

        let mut cancel = false;
        let mut confirm = false;

        egui::Window::new("Delete Profile")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Delete profile '{}' ?", profile.name));
                ui.small(format!("Profile file: {}", profiles::profiles_path_text()));
                if profile.dbnest_instance_id.is_some() {
                    ui.colored_label(
                        theme::WARNING,
                        "This only removes the RustLens profile. It does not remove the dbnest database instance.",
                    );
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "Cancel").clicked() {
                        cancel = true;
                    }
                    if theme::danger_button(ui, "Delete profile").clicked() {
                        confirm = true;
                    }
                });
            });

        if cancel {
            self.pending_profile_delete = None;
            self.detail_status = "Profile deletion cancelled.".to_string();
        }
        if confirm {
            self.delete_profile(&profile);
        }
    }
}
